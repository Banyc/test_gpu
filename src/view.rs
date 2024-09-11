use std::sync::Arc;

use anyhow::Context;

use crate::gpu::{adapter, device};

#[derive(Debug)]
pub struct View {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    draw: Box<dyn Draw>,
}
impl View {
    pub async fn new(
        window: Arc<winit::window::Window>,
        instance: &wgpu::Instance,
        mut draw: Box<dyn Draw>,
    ) -> anyhow::Result<Self> {
        let surface = instance.create_surface(window.clone())?;
        let adapter = adapter(instance, Some(&surface))
            .await
            .context("no adapter")?;
        let (device, queue) = device(&adapter).await?;
        let args = InitArgs {
            device: &device,
            surface: &surface,
            adapter: &adapter,
        };
        draw.init(args);
        Ok(Self {
            window,
            surface,
            adapter,
            device,
            queue,
            draw,
        })
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let Some(config) = self
            .surface
            .get_default_config(&self.adapter, size.width, size.height)
        else {
            return;
        };
        self.surface.configure(&self.device, &config);
    }

    pub fn draw(&mut self) -> anyhow::Result<()> {
        let frame = self.surface.get_current_texture()?;
        let desc = wgpu::TextureViewDescriptor::default();
        let view = frame.texture.create_view(&desc);
        let desc = wgpu::CommandEncoderDescriptor { label: None };
        let mut command = self.device.create_command_encoder(&desc);
        let args = DrawArgs {
            command: &mut command,
            view,
            device: &self.device,
            queue: &self.queue,
        };
        self.draw.draw(args);
        self.queue.submit([command.finish()]);
        frame.present();
        Ok(())
    }
}

#[derive(Debug)]
pub struct InitArgs<'a> {
    pub device: &'a wgpu::Device,
    pub surface: &'a wgpu::Surface<'a>,
    pub adapter: &'a wgpu::Adapter,
}

#[derive(Debug)]
pub struct DrawArgs<'a> {
    pub command: &'a mut wgpu::CommandEncoder,
    pub view: wgpu::TextureView,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

pub trait Draw: core::fmt::Debug + Sync + Send {
    fn init(&mut self, args: InitArgs<'_>);
    fn draw(&mut self, args: DrawArgs<'_>);
}
