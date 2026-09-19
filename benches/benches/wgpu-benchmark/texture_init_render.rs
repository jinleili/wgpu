use std::{sync::mpsc, time::Instant};

use nanorand::{Rng, WyRand};
use wgpu_benchmark::{iter, iter_many, BenchmarkContext, SubBenchResult};

use crate::DeviceState;

#[derive(Clone, Copy)]
enum Scenario {
    Shared,
    Overlapping,
    Discard,
    Control,
}

pub fn shared(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    run(ctx, Scenario::Shared)
}

pub fn overlapping(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    run(ctx, Scenario::Overlapping)
}

pub fn discard(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    run(ctx, Scenario::Discard)
}

pub fn control(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    run(ctx, Scenario::Control)
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const PIXEL: [u8; 4] = [64, 128, 192, 255];

fn run(ctx: BenchmarkContext, scenario: Scenario) -> anyhow::Result<Vec<SubBenchResult>> {
    let state = DeviceState::new();
    let device = &state.device;
    let count = if ctx.is_test() { 8 } else { 1_000 };
    let passes = if ctx.is_test() { 2 } else { 10 };
    let control = matches!(scenario, Scenario::Control);
    let mixed = matches!(scenario, Scenario::Overlapping | Scenario::Discard);
    let discard = matches!(scenario, Scenario::Discard);
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &if control {
            Vec::new()
        } else {
            (0..2)
                .map(|binding| wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                })
                .collect()
        },
    });
    let shader = device.create_shader_module(wgpu::include_wgsl!("texture_init_render.wgsl"));
    let make_pipeline = |layout: Option<&wgpu::BindGroupLayout>, entry_point| {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layout.map(Some).into_iter().collect::<Vec<_>>(),
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                targets: &[Some(FORMAT.into())],
            }),
            multiview_mask: None,
            cache: None,
        })
    };
    let producer = make_pipeline(None, "solid");
    let consumer = make_pipeline(Some(&layout), if control { "solid" } else { "sample_pair" });
    let make_texture = |usage, mip_level_count| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage,
            view_formats: &[],
        })
    };
    let mut attachment_views = Vec::new();
    let mut pairs = Vec::new();
    for _ in 0..if control { 0 } else { count.min(16) } {
        let a = make_texture(
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            3,
        );
        let b = make_texture(
            wgpu::TextureUsages::TEXTURE_BINDING
                | if mixed {
                    wgpu::TextureUsages::COPY_DST
                } else {
                    wgpu::TextureUsages::RENDER_ATTACHMENT
                },
            3,
        );
        for mip in 0..3 {
            let view = wgpu::TextureViewDescriptor {
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            };
            attachment_views.push(a.create_view(&view));
            if mixed {
                let width = 4 >> mip;
                state.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        mip_level: mip,
                        ..b.as_image_copy()
                    },
                    &PIXEL.repeat((width * width) as usize),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(width * 4),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width,
                        height: width,
                        depth_or_array_layers: 1,
                    },
                );
            } else {
                attachment_views.push(b.create_view(&view));
            }
        }
        for base_mip_level in 0..if mixed { 2 } else { 1 } {
            let view = wgpu::TextureViewDescriptor {
                base_mip_level,
                mip_level_count: mixed.then_some(2),
                ..Default::default()
            };
            pairs.push([a.create_view(&view), b.create_view(&view)]);
        }
    }
    WyRand::new_seed(0x8BADF00D).shuffle(&mut pairs);
    let entries: Vec<_> = pairs
        .iter()
        .map(|pair| {
            core::array::from_fn::<_, 2, _>(|binding| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: wgpu::BindingResource::TextureView(&pair[binding]),
            })
        })
        .collect();
    let descriptors: Vec<_> = (0..count)
        .map(|i| wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: entries
                .get(i as usize % entries.len().max(1))
                .map_or(&[][..], |entries| &entries[..]),
        })
        .collect();
    let groups: Vec<_> = descriptors
        .iter()
        .map(|d| device.create_bind_group(d))
        .collect();
    let output = make_texture(
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        1,
    );
    let output_view = output.create_view(&Default::default());
    let encode = || {
        let mut encoder = device.create_command_encoder(&Default::default());
        let write_attachments = |encoder: &mut wgpu::CommandEncoder, store| {
            for view in &attachment_views {
                let attachments = [Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store,
                    },
                })];
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &attachments,
                    ..Default::default()
                });
                pass.set_pipeline(&producer);
                pass.draw(0..3, 0..1);
            }
        };
        write_attachments(&mut encoder, wgpu::StoreOp::Store);
        for index in 0..passes {
            if discard && index == passes / 2 {
                write_attachments(&mut encoder, wgpu::StoreOp::Discard);
            }
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &attachments,
                ..Default::default()
            });
            pass.set_pipeline(&consumer);
            for group in &groups {
                pass.set_bind_group(0, group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        encoder.finish()
    };
    for _ in 0..8 {
        state.queue.submit([encode()]);
        device.poll(wgpu::PollType::wait_indefinitely())?;
    }
    check_output(
        &state,
        &output,
        if discard { [32, 64, 96, 128] } else { PIXEL },
    )?;
    let mut results = iter_many(
        &ctx,
        vec!["Encoding".into(), "Submit".into()],
        "draws",
        count * passes,
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
    );
    let mut created = Vec::with_capacity(count as usize);
    results.push(iter(&ctx, "Create bind groups", "groups", count, || {
        let start = Instant::now();
        created.extend(descriptors.iter().map(|d| device.create_bind_group(d)));
        let elapsed = start.elapsed();
        created.clear();
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        elapsed
    }));
    Ok(results)
}

fn check_output(
    state: &DeviceState,
    output: &wgpu::Texture,
    expected: [u8; 4],
) -> anyhow::Result<()> {
    let buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = state.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        output.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    state.queue.submit([encoder.finish()]);
    let (send, receive) = mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        send.send(result).unwrap()
    });
    state.device.poll(wgpu::PollType::wait_indefinitely())?;
    receive.recv()??;
    let actual = buffer.get_mapped_range(..)?;
    anyhow::ensure!(
        actual.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 1),
        "render chain produced {:?}, expected {:?}",
        &*actual,
        expected
    );
    drop(actual);
    buffer.unmap();
    Ok(())
}
