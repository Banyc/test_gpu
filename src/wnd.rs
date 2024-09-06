use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{
    gpu::instance,
    view::{Draw, View},
};

#[derive(Debug)]
pub struct Wnd {
    draw: Option<Box<dyn Draw>>,
    view: Option<View>,
}
impl Wnd {
    pub fn new(draw: Box<dyn Draw>) -> Self {
        Self {
            view: None,
            draw: Some(draw),
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
        let view = View::new(window, &instance, self.draw.take().unwrap());
        let view = pollster::block_on(view).unwrap();
        self.view = Some(view);
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
                self.view.as_mut().unwrap().draw().unwrap();
            }
            WindowEvent::Resized(size) => {
                self.view.as_mut().unwrap().resize(size);
                self.view.as_ref().unwrap().window().request_redraw();
            }
            _ => (),
        }
    }
}
