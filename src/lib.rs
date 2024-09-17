pub mod camera;
pub mod compute;
pub mod delta_time;
pub mod gpu;
pub mod texture;
pub mod transform;
pub mod triangle;
pub mod wnd;

pub trait RenderApp: Draw + Resize + Update + core::fmt::Debug + Sync + Send {}

#[derive(Debug)]
pub struct RenderInitArgs<'a> {
    pub device: &'a wgpu::Device,
    pub surface: &'a wgpu::Surface<'a>,
    pub adapter: &'a wgpu::Adapter,
    pub queue: &'a wgpu::Queue,
    pub wnd_size: WndSize,
}
pub trait RenderInit: core::fmt::Debug {
    fn init(&self, args: RenderInitArgs<'_>) -> Box<dyn RenderApp>;
}

#[derive(Debug)]
pub struct DrawArgs<'a> {
    pub view: wgpu::TextureView,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}
pub trait Draw {
    fn draw(&mut self, args: DrawArgs<'_>) -> RenderNextStep;
}

#[derive(Debug)]
pub struct UpdateArgs {
    pub event: winit::event::WindowEvent,
}
pub trait Update {
    fn update(&mut self, args: UpdateArgs) -> RenderNextStep;
}

#[derive(Debug)]
pub struct ResizeArgs<'a> {
    pub device: &'a wgpu::Device,
    pub size: WndSize,
}
pub trait Resize {
    fn resize(&mut self, args: ResizeArgs<'_>) -> RenderNextStep;
}

#[derive(Debug, Clone, Copy)]
pub struct WndSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct RenderNextStep {
    pub should_request_redraw: bool,
}
