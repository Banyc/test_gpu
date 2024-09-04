use wgpu::util::DeviceExt;

pub fn instance() -> wgpu::Instance {
    wgpu::Instance::default()
}
/// handle to graphics card
pub async fn adapter(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Option<wgpu::Adapter> {
    let options = wgpu::RequestAdapterOptions {
        compatible_surface: surface,
        ..Default::default()
    };
    instance.request_adapter(&options).await
}
#[tokio::test]
async fn test_adapter() {
    let instance = instance();
    let adapter = adapter(&instance, None).await.unwrap();
    println!("{:?}", adapter.get_info());
}
pub async fn device(adapter: &wgpu::Adapter) -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
    let trace_path = None;
    let desc = wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        memory_hints: wgpu::MemoryHints::Performance,
    };
    let device = adapter.request_device(&desc, trace_path).await?;
    Ok(device)
}

pub async fn compute<I, G, O>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shader: wgpu::ShaderSource<'_>,
    global: &G,
    input_seq: &[I],
    output_seq: &mut [O],
) where
    I: bytemuck::Pod,
    G: bytemuck::Pod,
    O: bytemuck::Pod,
{
    let desc = wgpu::ShaderModuleDescriptor {
        label: None,
        source: shader,
    };
    let shader = device.create_shader_module(desc);
    let out_buf_size = core::mem::size_of_val(output_seq);
    let out_buf_size = wgpu::BufferAddress::try_from(out_buf_size).unwrap();
    let desc = wgpu::BufferDescriptor {
        label: Some("staging buf"),
        size: out_buf_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    let staging_buf = device.create_buffer(&desc);
    let desc = wgpu::BufferDescriptor {
        label: Some("out buf"),
        size: out_buf_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    };
    let out_buf = device.create_buffer(&desc);
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("in buf"),
        contents: bytemuck::cast_slice(input_seq),
        usage: wgpu::BufferUsages::STORAGE,
    };
    let in_buf = device.create_buffer_init(&desc);
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("global buf"),
        contents: bytemuck::bytes_of(global),
        usage: wgpu::BufferUsages::UNIFORM,
    };
    let global_buf = device.create_buffer_init(&desc);

    let desc = wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &shader,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    };
    let pipeline = device.create_compute_pipeline(&desc);
    let layout = pipeline.get_bind_group_layout(0);
    let desc = wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: in_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: out_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: global_buf.as_entire_binding(),
            },
        ],
    };
    let bind_group = device.create_bind_group(&desc);

    let desc = wgpu::CommandEncoderDescriptor { label: None };
    let mut command = device.create_command_encoder(&desc);
    {
        let desc = wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        };
        let mut pass = command.begin_compute_pass(&desc);
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.insert_debug_marker("compute");
        pass.dispatch_workgroups(input_seq.len().try_into().unwrap(), 1, 1);
    }
    command.copy_buffer_to_buffer(&out_buf, 0, &staging_buf, 0, out_buf_size);

    queue.submit([command.finish()]);

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let staging_slice = staging_buf.slice(..);
    staging_slice.map_async(wgpu::MapMode::Read, move |v| tx.try_send(v).unwrap());

    let res = rx.recv().await.unwrap();
    res.unwrap();

    let data = staging_slice.get_mapped_range();
    let result = bytemuck::cast_slice(&data);
    output_seq.copy_from_slice(result);

    drop(data);
    staging_buf.unmap();
}
#[tokio::test]
async fn test_compute() {
    use std::sync::Arc;

    use crate::shaders::U32_IDENTITY_WGSL;

    let instance = instance();
    let adapter = adapter(&instance, None).await.unwrap();
    let (device, queue) = device(&adapter).await.unwrap();
    let device = Arc::new(device);
    std::thread::spawn({
        let device = Arc::clone(&device);
        move || loop {
            device.poll(wgpu::Maintain::Wait);
        }
    });
    let identity_src = wgpu::ShaderSource::Wgsl(U32_IDENTITY_WGSL.into());

    let input_seq: [u32; 3] = [0, 1, 2];
    let mut output_seq = [0, 0, 0];
    compute(
        &device,
        &queue,
        identity_src,
        &0,
        &input_seq,
        &mut output_seq,
    )
    .await;
    assert_eq!(input_seq, output_seq);
}
