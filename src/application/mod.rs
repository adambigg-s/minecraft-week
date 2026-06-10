mod input;

use std::{env, sync};

use winit::{application, dpi, event_loop, window};

use crate::render;

pub trait Application
where
    Self: Sized,
{
    fn setup(
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    ) -> anyhow::Result<Self>;
}

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

#[derive(bon::Builder, Debug)]
struct State<Inner> {
    window: sync::Arc<window::Window>,
    gfx_context: render::GfxContext,
    gfx_render: render::GfxRenderer,
    input: input::Input,
    inner_state: Inner,
}

impl<Inner> State<Inner>
where
    Inner: Application,
{
    async fn new(window: sync::Arc<window::Window>) -> anyhow::Result<Self> {
        let mut gfx_context = pollster::block_on(render::GfxContext::new(sync::Arc::clone(&window)))?;

        let mut gfx_render = render::GfxRenderer::new(&gfx_context)?;

        let input = input::Input::new();

        let inner_state = Inner::setup(&mut gfx_context, &mut gfx_render)?;

        Ok(Self { window, gfx_context, gfx_render, input, inner_state })
    }
}

#[derive(bon::Builder, Debug)]
struct ApplicationRunner<Inner> {
    inner: Option<State<Inner>>,
}

impl<Inner> application::ApplicationHandler for ApplicationRunner<Inner>
where
    Inner: Application,
{
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        log::warn!("Application resume requested");

        if self.inner.is_some() {
            log::error!("False resume");
            return;
        }

        let window_attrs = window::WindowAttributes::default()
            .with_inner_size(dpi::PhysicalSize::new(1920, 1080))
            .with_title("Minecraft-week game");
        let window = sync::Arc::new(event_loop.create_window(window_attrs).unwrap());

        self.inner = Some(pollster::block_on(State::new(window)).unwrap());
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
