use test_gpu::{triangle::DrawTriangle, wnd::Wnd};
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let event_loop = EventLoop::builder().build()?;
    let draw = DrawTriangle::new();
    let mut app = Wnd::new(Box::new(draw));
    event_loop.run_app(&mut app)?;
    Ok(())
}
