use std::time::Instant;

use nanorand::{Rng, WyRand};
use wgpu_benchmark::{iter, iter_many, BenchmarkContext, SubBenchResult};

use crate::DeviceState;

enum Scenario {
    FullView,
    PartialView,
    Attachment,
    Mixed,
    Unsubmitted,
    NoTextures,
    SharedTextures,
    OverlappingViews,
}

pub fn run_bench(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    let state = DeviceState::new();
    let device = &state.device;
    let count = if ctx.is_test() { 8 } else { 1_000 };
    let repeats = if ctx.is_test() { 1 } else { 10 };
    let mut results = Vec::new();

    for scenario in [
        Scenario::FullView,
        Scenario::PartialView,
        Scenario::Attachment,
        Scenario::Mixed,
        Scenario::Unsubmitted,
        Scenario::NoTextures,
        Scenario::SharedTextures,
        Scenario::OverlappingViews,
    ] {
        let (name, textures_per_group, attachments) = match scenario {
            Scenario::FullView => ("Full view", 2, 0),
            Scenario::PartialView => ("Partial view", 2, 0),
            Scenario::Attachment => ("Attachment", 2, 2),
            Scenario::Mixed => ("Mixed", 2, 1),
            Scenario::Unsubmitted => ("Unsubmitted", 2, 0),
            Scenario::NoTextures => ("No textures", 0, 0),
            Scenario::SharedTextures => ("Shared attachments", 2, 2),
            Scenario::OverlappingViews => ("Overlapping mixed views", 2, 1),
        };
        let partial = matches!(scenario, Scenario::PartialView);
        device.poll(wgpu::PollType::wait_indefinitely())?;
        let shared = matches!(
            scenario,
            Scenario::SharedTextures | Scenario::OverlappingViews
        );
        let overlapping = matches!(scenario, Scenario::OverlappingViews);
        let texture_count = if shared { count.min(16) } else { count } * textures_per_group;
        let source = if texture_count == 0 {
            r#"
                @group(0) @binding(2) var<storage, read_write> result: u32;
                @compute @workgroup_size(1) fn main() { result = 0u; }
            "#
        } else {
            r#"
                @group(0) @binding(0) var a: texture_2d_array<u32>;
                @group(0) @binding(1) var b: texture_2d_array<u32>;
                @group(0) @binding(2) var<storage, read_write> result: u32;
                @compute @workgroup_size(1) fn main() {
                    result = textureLoad(a, vec2<u32>(0), 0, 0).x
                           + textureLoad(b, vec2<u32>(0), 0, 0).x;
                }
            "#
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let layout = pipeline.get_bind_group_layout(0);
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let textures: Vec<_> = (0..texture_count)
            .map(|i| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: 4,
                        height: 4,
                        depth_or_array_layers: 2,
                    },
                    mip_level_count: 3,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R32Uint,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | if i % 2 < attachments {
                            wgpu::TextureUsages::RENDER_ATTACHMENT
                        } else {
                            wgpu::TextureUsages::empty()
                        },
                    view_formats: &[],
                })
            })
            .collect();
        let mut pairs: Vec<_> = textures
            .chunks_exact(2)
            .flat_map(|pair| {
                (0..if overlapping { 2 } else { 1 }).map(move |view| {
                    let desc = wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        base_mip_level: if overlapping {
                            view
                        } else {
                            u32::from(partial)
                        },
                        mip_level_count: if overlapping {
                            Some(2)
                        } else {
                            partial.then_some(1)
                        },
                        base_array_layer: u32::from(partial),
                        array_layer_count: partial.then_some(1),
                        ..Default::default()
                    };
                    [pair[0].create_view(&desc), pair[1].create_view(&desc)]
                })
            })
            .collect();
        // Keep each mixed pair together when shuffling.
        WyRand::new_seed(0x8BADF00D).shuffle(&mut pairs);
        let buffer_entries = [wgpu::BindGroupEntry {
            binding: 2,
            resource: output.as_entire_binding(),
        }];
        let texture_entries: Vec<_> = pairs
            .iter()
            .map(|pair| {
                [
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&pair[0]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&pair[1]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: output.as_entire_binding(),
                    },
                ]
            })
            .collect();
        let descriptors: Vec<_> = (0..count)
            .map(|i| wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: texture_entries
                    .get(i as usize % texture_entries.len().max(1))
                    .map_or(&buffer_entries[..], |entries| &entries[..]),
            })
            .collect();
        let create_groups = |groups: &mut Vec<wgpu::BindGroup>| {
            groups.extend(
                descriptors
                    .iter()
                    .map(|desc| device.create_bind_group(desc)),
            );
        };
        let mut groups = Vec::with_capacity(count as usize);
        create_groups(&mut groups);
        let encode = || {
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(&pipeline);
                for _ in 0..repeats {
                    for group in &groups {
                        pass.set_bind_group(0, group, &[]);
                        pass.dispatch_workgroups(1, 1, 1);
                    }
                }
            }
            encoder.finish()
        };
        if matches!(scenario, Scenario::Unsubmitted) {
            // Dropping the recorded work must leave device initialization unchanged.
            results.push(iter(&ctx, name, "dispatches", count * repeats, || {
                let start = Instant::now();
                let commands = encode();
                let elapsed = start.elapsed();
                drop(commands);
                elapsed
            }));
        } else {
            for _ in 0..2 {
                state.queue.submit([encode()]);
                device.poll(wgpu::PollType::wait_indefinitely())?;
            }
            results.extend(iter_many(
                &ctx,
                vec![format!("{name}: encoding"), format!("{name}: submit")],
                "dispatches",
                count * repeats,
                || {
                    let start = Instant::now();
                    let commands = encode();
                    let encoding = start.elapsed();
                    let start = Instant::now();
                    state.queue.submit([commands]);
                    let submit = start.elapsed();
                    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                    vec![encoding, submit]
                },
            ));
        }
        if matches!(scenario, Scenario::FullView) {
            let mut created_groups = Vec::with_capacity(count as usize);
            results.push(iter(&ctx, "Create bind groups", "groups", count, || {
                let start = Instant::now();
                create_groups(&mut created_groups);
                let elapsed = start.elapsed();
                created_groups.clear();
                device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                elapsed
            }));
        }
    }
    Ok(results)
}
