use wgpu::*;
use wgpu_test::{
    apply, fail, gpu_test, GpuTestConfiguration, GpuTestInitializer, TestParameters, TestingContext,
};

use super::map_and_read;

pub fn all_tests(vec: &mut Vec<GpuTestInitializer>) {
    vec.extend([
        PARTIAL_VIEW,
        DROPPED_COMMAND_BUFFER,
        DISCARD_SAME_ENCODER,
        DISCARD_SEPARATE_SUBMISSIONS,
        REORDERED_SUBMISSIONS,
        OVERLAPPING_VIEWS,
        DESTROYED_TEXTURE,
    ]);
}

fn parameters() -> TestParameters {
    TestParameters::default()
        .downlevel_flags(DownlevelFlags::COMPUTE_SHADERS)
        .limits(Limits::downlevel_defaults())
}

#[apply(gpu_test!)]
static PARTIAL_VIEW: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(parameters())
    .run_async(|ctx| async move {
        for extra_usage in [TextureUsages::empty(), TextureUsages::RENDER_ATTACHMENT] {
            let case = TextureBindingCase::new(&ctx, extra_usage);
            case.write(0, 0, 7);
            let narrow_group = case.bind_group(&TextureViewDescriptor {
                dimension: Some(TextureViewDimension::D2Array),
                mip_level_count: Some(1),
                array_layer_count: Some(1),
                ..Default::default()
            });
            // Exercise the same view across submissions before widening its range.
            for round in 0..3 {
                ctx.queue.submit([case.encode_read(&narrow_group)]);
                case.assert_contents(1, 1, |_, _| 7, &format!("narrow view, round {round}"))
                    .await;
            }

            let full_group = case.bind_group(&TextureViewDescriptor::default());
            for round in 0..3 {
                let commands = case.encode_read(&full_group);
                if round == 2 {
                    // A queue write after encoding must survive submission-time initialization.
                    case.write(1, 1, 9);
                }
                ctx.queue.submit([commands]);
                case.assert_contents(
                    case.texture.mip_level_count(),
                    case.texture.depth_or_array_layers(),
                    |mip, layer| match (mip, layer) {
                        (0, 0) => 7,
                        (1, 1) if round == 2 => 9,
                        _ => 0,
                    },
                    &format!("full view, round {round}"),
                )
                .await;
            }
        }
    });

#[apply(gpu_test!)]
static DROPPED_COMMAND_BUFFER: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(parameters())
    .run_async(|ctx| async move {
        let case = TextureBindingCase::new(&ctx, TextureUsages::empty());
        let group = case.bind_group(&TextureViewDescriptor::default());
        drop(case.encode_read(&group));
        ctx.queue.submit([case.encode_read(&group)]);
        case.assert_contents(
            case.texture.mip_level_count(),
            case.texture.depth_or_array_layers(),
            |_, _| 0,
            "read after dropping unsubmitted commands",
        )
        .await;
    });

#[apply(gpu_test!)]
static DISCARD_SAME_ENCODER: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(parameters())
    .run_async(|ctx| check_discard(ctx, true));

#[apply(gpu_test!)]
static DISCARD_SEPARATE_SUBMISSIONS: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(parameters())
    .run_async(|ctx| check_discard(ctx, false));

#[apply(gpu_test!)]
static REORDERED_SUBMISSIONS: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(parameters())
    .run_async(|ctx| async move {
        let case = TextureBindingCase::new(&ctx, TextureUsages::RENDER_ATTACHMENT);
        case.write(1, 1, 19);
        let group = case.bind_group(&TextureViewDescriptor::default());
        let read = case.encode_read(&group);
        let mut discard = ctx.device.create_command_encoder(&Default::default());
        case.record_discard(&mut discard, 1, 1);
        ctx.queue.submit([discard.finish(), read]);
        case.assert_contents(
            case.texture.mip_level_count(),
            case.texture.depth_or_array_layers(),
            |_, _| 0,
            "submit discard before an earlier encoded read",
        )
        .await;
    });

#[apply(gpu_test!)]
static OVERLAPPING_VIEWS: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(parameters())
    .run_async(|ctx| async move {
        let case = TextureBindingCase::new(&ctx, TextureUsages::RENDER_ATTACHMENT);
        case.write(2, 1, 23);
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        for (mip, levels, layer, layers) in [(0, 2, 0, 1), (1, 2, 1, 1), (0, 3, 0, 2)] {
            let group = case.bind_group(&TextureViewDescriptor {
                dimension: Some(TextureViewDimension::D2Array),
                base_mip_level: mip,
                mip_level_count: Some(levels),
                base_array_layer: layer,
                array_layer_count: Some(layers),
                ..Default::default()
            });
            case.record_read(&mut encoder, &group);
        }
        ctx.queue.submit([encoder.finish()]);
        case.assert_contents(
            case.texture.mip_level_count(),
            case.texture.depth_or_array_layers(),
            |mip, layer| if (mip, layer) == (2, 1) { 23 } else { 0 },
            "overlapping views from distinct groups",
        )
        .await;
    });

async fn check_discard(ctx: TestingContext, same_encoder: bool) {
    let case = TextureBindingCase::new(&ctx, TextureUsages::RENDER_ATTACHMENT);
    let sentinel = |mip, layer| 7 + mip * case.texture.depth_or_array_layers() + layer;
    for mip in 0..case.texture.mip_level_count() {
        for layer in 0..case.texture.depth_or_array_layers() {
            case.write(mip, layer, sentinel(mip, layer));
        }
    }
    let group = case.bind_group(&TextureViewDescriptor::default());
    for round in 0..2 {
        ctx.queue.submit([case.encode_read(&group)]);
        case.assert_contents(
            case.texture.mip_level_count(),
            case.texture.depth_or_array_layers(),
            sentinel,
            &format!("before discard, round {round}"),
        )
        .await;
    }

    let target_mip = 1;
    let target_layer = 1;
    let mut encoder = ctx.device.create_command_encoder(&Default::default());
    if same_encoder {
        case.record_read(&mut encoder, &group);
    }
    case.record_discard(&mut encoder, target_mip, target_layer);
    if !same_encoder {
        ctx.queue.submit([encoder.finish()]);
        encoder = ctx.device.create_command_encoder(&Default::default());
    }
    case.record_read(&mut encoder, &group);
    ctx.queue.submit([encoder.finish()]);
    case.assert_contents(
        case.texture.mip_level_count(),
        case.texture.depth_or_array_layers(),
        |mip, layer| {
            if (mip, layer) == (target_mip, target_layer) {
                0
            } else {
                sentinel(mip, layer)
            }
        },
        "read after discard",
    )
    .await;
}

#[apply(gpu_test!)]
static DESTROYED_TEXTURE: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(parameters())
    .run_async(|ctx| async move {
        let case = TextureBindingCase::new(&ctx, TextureUsages::empty());
        let group = case.bind_group(&TextureViewDescriptor::default());
        for _ in 0..2 {
            ctx.queue.submit([case.encode_read(&group)]);
        }
        ctx.async_poll(PollType::wait_indefinitely()).await.unwrap();
        case.texture.destroy();
        fail(
            &ctx.device,
            || ctx.queue.submit([case.encode_read(&group)]),
            Some("destroyed"),
        );
    });

struct TextureBindingCase<'a> {
    ctx: &'a TestingContext,
    texture: Texture,
    other_view: TextureView,
    pipeline: ComputePipeline,
    output: Buffer,
    readback: Buffer,
}

impl<'a> TextureBindingCase<'a> {
    fn new(ctx: &'a TestingContext, extra_usage: TextureUsages) -> Self {
        let texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some("texture binding init"),
            size: Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 2,
            },
            mip_level_count: 3,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Uint,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | extra_usage,
            view_formats: &[],
        });
        let other_view = ctx
            .device
            .create_texture(&TextureDescriptor {
                label: Some("non-attachment in the same group"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Uint,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor::default());
        let shader = ctx.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(
                r#"
                @group(0) @binding(0) var tex: texture_2d_array<u32>;
                @group(0) @binding(1) var<storage, read_write> result: array<u32>;
                @group(0) @binding(2) var other: texture_2d<u32>;
                @compute @workgroup_size(1) fn main() {
                    var index = 0u;
                    for (var mip = 0u; mip < textureNumLevels(tex); mip++) {
                        let size = textureDimensions(tex, mip);
                        for (var layer = 0u; layer < textureNumLayers(tex); layer++) {
                            for (var y = 0u; y < size.y; y++) {
                                for (var x = 0u; x < size.x; x++) {
                                    result[index] = textureLoad(tex, vec2<u32>(x, y), layer, mip).x
                                                  + textureLoad(other, vec2<u32>(0), 0).x;
                                    index++;
                                }
                            }
                        }
                    }
                }
                "#
                .into(),
            ),
        });
        let pipeline = ctx
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });
        let texel_count: u32 = (0..texture.mip_level_count())
            .map(|mip| {
                (texture.width() >> mip)
                    * (texture.height() >> mip)
                    * texture.depth_or_array_layers()
            })
            .sum();
        let output = ctx.device.create_buffer(&BufferDescriptor {
            label: None,
            size: u64::from(texel_count) * size_of::<u32>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = ctx.device.create_buffer(&BufferDescriptor {
            label: None,
            size: output.size(),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            ctx,
            texture,
            other_view,
            pipeline,
            output,
            readback,
        }
    }

    fn bind_group(&self, view: &TextureViewDescriptor) -> BindGroup {
        self.ctx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&self.texture.create_view(view)),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.output.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&self.other_view),
                },
            ],
        })
    }

    fn record_read(&self, encoder: &mut CommandEncoder, group: &BindGroup) {
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&self.output, 0, &self.readback, 0, self.output.size());
    }

    fn encode_read(&self, group: &BindGroup) -> CommandBuffer {
        let mut encoder = self.ctx.device.create_command_encoder(&Default::default());
        self.record_read(&mut encoder, group);
        encoder.finish()
    }

    fn record_discard(&self, encoder: &mut CommandEncoder, mip: u32, layer: u32) {
        let view = self.texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2),
            base_mip_level: mip,
            mip_level_count: Some(1),
            base_array_layer: layer,
            array_layer_count: Some(1),
            ..Default::default()
        });
        let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    // Preserve the sentinel so an omitted discard fixup remains observable.
                    load: LoadOp::Load,
                    store: StoreOp::Discard,
                },
            })],
            ..Default::default()
        });
    }

    fn write(&self, mip: u32, layer: u32, value: u32) {
        let width = self.texture.width() >> mip;
        let height = self.texture.height() >> mip;
        let data = vec![value; (width * height) as usize];
        self.ctx.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: mip,
                origin: Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: TextureAspect::All,
            },
            bytemuck::cast_slice(&data),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * size_of::<u32>() as u32),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    async fn assert_contents(
        &self,
        mip_count: u32,
        layer_count: u32,
        expected: impl Fn(u32, u32) -> u32,
        scenario: &str,
    ) {
        let bytes = map_and_read(self.ctx, &self.readback).await;
        let texels = bytemuck::cast_slice::<u8, u32>(&bytes);
        let mut index = 0;
        for mip in 0..mip_count {
            for layer in 0..layer_count {
                for y in 0..(self.texture.height() >> mip) {
                    for x in 0..(self.texture.width() >> mip) {
                        assert_eq!(
                            texels[index],
                            expected(mip, layer),
                            "{scenario}: usage={:?}, mip={mip}, layer={layer}, x={x}, y={y}",
                            self.texture.usage(),
                        );
                        index += 1;
                    }
                }
            }
        }
        self.readback.unmap();
    }
}
