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
        required_features: wgpu::Features::empty() | wgpu::Features::POLYGON_MODE_LINE,
        required_limits: wgpu::Limits::downlevel_defaults(),
        memory_hints: wgpu::MemoryHints::Performance,
    };
    let device = adapter.request_device(&desc, trace_path).await?;
    Ok(device)
}
