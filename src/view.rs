use std::sync::Arc;

use anyhow::Context;

use crate::gpu::{adapter, device};

pub struct View {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl View {
    pub async fn new(
        window: Arc<winit::window::Window>,
        instance: &wgpu::Instance,
    ) -> anyhow::Result<Self> {
        let surface = instance.create_surface(window.clone())?;
        let adapter = adapter(instance, Some(&surface))
            .await
            .context("no adapter")?;
        let (device, queue) = device(&adapter).await?;
        Ok(Self {
            window,
            surface,
            adapter,
            device,
            queue,
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

    pub fn draw(&self) -> anyhow::Result<()> {
        let frame = self.surface.get_current_texture()?;
        let desc = wgpu::TextureViewDescriptor::default();
        let view = frame.texture.create_view(&desc);
        let desc = wgpu::CommandEncoderDescriptor { label: None };
        let mut command = self.device.create_command_encoder(&desc);
        self.queue.submit([command.finish()]);
        Ok(())
    }
}
