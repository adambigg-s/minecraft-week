use std::{env, sync};

use winit::{application, event_loop, window};

pub trait Application {}

fn init() {
    unsafe { env::set_var("RUST_LOG", "info") };
    env_logger::init();
    log::warn!("Application started");
}

fn cleanup() {
    log::warn!("Application terminated successfully");
}

pub fn run<Inner>() -> anyhow::Result<()>
where
    Inner: Application,
{
    init();
    event_loop::EventLoop::new()?.run_app(&mut ApplicationRunner::<Inner> { inner: None })?;
    cleanup();
    Ok(())
}

struct State<Inner> {
    inner_state: Inner,
}

impl<Inner> State<Inner>
where
    Inner: Application,
{
    async fn new(window: sync::Arc<window::Window>) -> anyhow::Result<Self> {
        todo!()
    }
}

struct ApplicationRunner<Inner> {
    inner: Option<State<Inner>>,
}

impl<Inner> application::ApplicationHandler for ApplicationRunner<Inner>
where
    Inner: Application,
{
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        todo!()
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        todo!()
    }
}
