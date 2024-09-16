use std::sync::Arc;

use anyhow::Context;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{
    gpu::{adapter, device, instance},
    DrawArgs, RenderApp, RenderInit, RenderInitArgs, WndSize,
};

#[derive(Debug)]
pub struct Wnd {
    app: Option<Box<dyn RenderInit>>,
    viewer: Option<ActiveWnd>,
}
impl Wnd {
    pub fn new(view: Box<dyn RenderInit>) -> Self {
        Self {
            viewer: None,
            app: Some(view),
        }
    }
}
impl ApplicationHandler for Wnd {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        tracing::info!("resumed");
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let window = Arc::new(window);
        let instance = instance();
        let view = ActiveWnd::new(window, &instance, self.app.take().unwrap());
        let view = pollster::block_on(view).unwrap();
        self.viewer = Some(view);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                tracing::info!("redraw requested");
                self.viewer.as_mut().unwrap().draw().unwrap();
            }
            WindowEvent::Resized(size) => {
                self.viewer.as_mut().unwrap().resize(size);
            }
            x => self.viewer.as_mut().unwrap().update(x),
        }
    }
}

#[derive(Debug)]
struct ActiveWnd {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    app: Box<dyn RenderApp>,
}
impl ActiveWnd {
    pub async fn new<A>(
        window: Arc<winit::window::Window>,
        instance: &wgpu::Instance,
        app: Box<A>,
    ) -> anyhow::Result<Self>
    where
        A: RenderInit + ?Sized,
    {
        let size = window.inner_size();
        let size = WndSize {
            width: size.width,
            height: size.height,
        };
        let surface = instance.create_surface(window.clone())?;
        let adapter = adapter(instance, Some(&surface))
            .await
            .context("no adapter")?;
        let (device, queue) = device(&adapter).await?;
        let args = RenderInitArgs {
            device: &device,
            surface: &surface,
            adapter: &adapter,
            queue: &queue,
            wnd_size: size,
        };
        let app = app.init(args);
        Ok(Self {
            window,
            surface,
            adapter,
            device,
            queue,
            app,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let Some(config) = self
            .surface
            .get_default_config(&self.adapter, size.width, size.height)
        else {
            return;
        };
        self.surface.configure(&self.device, &config);
        self.window.request_redraw();
        let args = WndSize {
            width: size.width,
            height: size.height,
        };
        self.app.resize(args);
    }

    pub fn update(&mut self, event: winit::event::WindowEvent) {
        self.app.update(event);
    }

    pub fn draw(&mut self) -> anyhow::Result<()> {
        let frame = self.surface.get_current_texture()?;
        let desc = wgpu::TextureViewDescriptor::default();
        let view = frame.texture.create_view(&desc);
        let args = DrawArgs {
            view,
            device: &self.device,
            queue: &self.queue,
        };
        self.app.draw(args);
        frame.present();
        Ok(())
    }
}
