use test_gpu::wnd::App;
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let event_loop = EventLoop::builder().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
