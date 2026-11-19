//! Tests of [`wgpu::Backend::Noop`].

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;

#[test]
fn device_is_not_available_by_default() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::NOOP,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect_err("noop backend adapter present when it should not be");
}

#[test]
fn device_is_available_when_requested() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::NOOP,
        backend_options: wgpu::BackendOptions {
            noop: wgpu::NoopBackendOptions::enabled(),
            ..Default::default()
        },
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("noop backend adapter absent when it should be");
}

#[test]
fn device_and_buffers() {
    let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());

    // Demonstrate that creating and *writing* to a buffer succeeds.
    // This also involves creation of a staging buffer.
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("hello world"),
        size: 8,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    assert_eq!(buffer.size(), 8);
    queue.write_buffer(&buffer, 0, &[1, 2, 3, 4]);
    queue.write_buffer(&buffer, 4, &[5, 6, 7, 8]);

    // Demonstrate that we can read back data from the buffer.
    // This also involves copy_buffer_to_buffer().
    let done: Arc<AtomicBool> = Arc::default();
    let done2 = done.clone();
    wgpu::util::DownloadBuffer::read_buffer(&device, &queue, &buffer.slice(..), move |result| {
        assert_eq!(*result.unwrap(), [1, 2, 3, 4, 5, 6, 7, 8],);
        done.store(true, Relaxed);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert!(done2.load(Relaxed));
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn async_pipeline_executor_owns_inputs_and_completes_once() {
    use futures_lite::future::poll_once;
    use std::sync::mpsc;

    #[derive(Debug)]
    struct Queue(mpsc::Sender<Box<dyn FnOnce() + Send>>);
    impl wgpu::PipelineExecutor for Queue {
        fn execute(&self, task: Box<dyn FnOnce() + Send>) {
            self.0.send(task).unwrap();
        }
    }

    let (tx, jobs) = mpsc::channel();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::NOOP,
        backend_options: wgpu::BackendOptions {
            noop: wgpu::NoopBackendOptions::enabled(),
            ..Default::default()
        },
        pipeline_executor: Some(Box::new(Queue(tx))),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, _queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let (render, compute, abandoned, rejected, lost) = {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                "@vertex fn vs(@location(0) p: vec4f) -> @builtin(position) vec4f { return p; }
                 @fragment fn fs() -> @location(0) vec4f { return vec4f(1); }
                 @compute @workgroup_size(1) fn cs() {}"
                    .into(),
            ),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor::default());
        let label = String::from("queued pipeline");
        let entry = String::from("cs");
        let compute = wgpu::ComputePipelineDescriptor {
            label: Some(&label),
            module: &module,
            layout: Some(&layout),
            entry_point: Some(&entry),
            compilation_options: Default::default(),
            cache: None,
        };
        let render = wgpu::RenderPipelineDescriptor {
            label: Some(&label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4],
                })],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            }),
            multiview_mask: None,
            cache: None,
        };
        let _ = device.create_compute_pipeline(&compute);
        let _ = device.create_render_pipeline(&render);
        assert!(matches!(jobs.try_recv(), Err(mpsc::TryRecvError::Empty)));
        (
            device.create_render_pipeline_async(&render),
            device.create_compute_pipeline_async(&compute),
            device.create_compute_pipeline_async(&compute),
            device.create_compute_pipeline_async(&compute),
            device.create_compute_pipeline_async(&wgpu::ComputePipelineDescriptor {
                entry_point: Some("missing"),
                ..compute
            }),
        )
    };

    let mut render = Box::pin(render);
    let mut compute = Box::pin(compute);
    assert!(pollster::block_on(poll_once(&mut render)).is_none());
    assert!(pollster::block_on(poll_once(&mut compute)).is_none());
    drop(abandoned);
    for _ in 0..3 {
        jobs.try_recv().unwrap()();
    }
    drop(jobs.try_recv().unwrap());
    assert!(pollster::block_on(render).is_ok());
    assert!(pollster::block_on(compute).is_ok());
    assert!(matches!(
        pollster::block_on(rejected),
        Err(wgpu::Error::Internal { .. })
    ));
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    // Device loss after enqueue takes precedence over validating the entry point.
    device.destroy();
    jobs.try_recv().unwrap()();
    assert!(pollster::block_on(lost).is_ok());
}
