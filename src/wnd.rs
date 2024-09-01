use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct App {
    window: Option<Window>,
}
impl App {
    pub fn new() -> Self {
        Self { window: None }
    }
}
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        tracing::info!("resumed");
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                tracing::info!("redraw requested");
            }
            _ => (),
        }
    }
}
