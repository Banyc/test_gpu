use test_gpu::{triangle::DrawTriangleInit, wnd::Wnd};
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let event_loop = EventLoop::builder().build()?;
    let app = DrawTriangleInit::new();
    let mut app = Wnd::new(Box::new(app));
    event_loop.run_app(&mut app)?;
    Ok(())
}
