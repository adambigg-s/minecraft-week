pub mod input;

use std::{env, sync};

use winit::{application, dpi, event, event_loop, keyboard, window};

use crate::render;

pub trait Application
where
    Self: Sized,
{
    fn config() -> Config;

    fn setup(
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    ) -> anyhow::Result<Self>;

    fn physics_frame(
        &mut self,
        input: &mut input::Input,
        gfx_context: &render::GfxContext,
        gfx_render: &render::GfxRenderer,
    );

    fn gfx_frame(
        &self,
        input: &input::Input,
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    );
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
pub struct Config {
    height: u32,
    width: u32,
    title: &'static str,
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

    pub fn config_changed(&mut self, width: u32, height: u32) {
        self.gfx_context.config.width = width;
        self.gfx_context.config.height = height;
        self.gfx_context.surface.configure(&self.gfx_context.device, &self.gfx_context.config);
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        self.inner_state.physics_frame(&mut self.input, &self.gfx_context, &self.gfx_render);
        Ok(())
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();
        self.inner_state.gfx_frame(&self.input, &mut self.gfx_context, &mut self.gfx_render);

        let output = match self.gfx_context.surface.get_current_texture() {
            | wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }
            | wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("Device lost");
            }
            | wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
        };

        let mut encoder = self
            .gfx_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Command encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output.texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.25, g: 0.25, b: 0.4, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.gfx_render.render(&mut render_pass);
        }

        self.gfx_context.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
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

        let config = Inner::config();
        let window_attrs = window::WindowAttributes::default()
            .with_inner_size(dpi::PhysicalSize::new(config.width, config.height))
            .with_title(config.title);
        let window = sync::Arc::new(event_loop.create_window(window_attrs).unwrap());

        self.inner = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn device_event(
        &mut self,
        _: &event_loop::ActiveEventLoop,
        _: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        let state = match &mut self.inner {
            | Some(state) => state,
            | None => {
                log::error!("False device event");
                return;
            }
        };

        if let event::DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            let (x, y) = &mut state.input.mouse_delta;
            *x += dx as f32;
            *y += dy as f32;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        _: window::WindowId,
        event: event::WindowEvent,
    ) {
        let state = match &mut self.inner {
            | Some(state) => state,
            | None => {
                log::error!("False window event");
                return;
            }
        };

        match event {
            | event::WindowEvent::ActivationTokenDone { .. } => todo!(),
            | event::WindowEvent::DroppedFile(_) => todo!(),
            | event::WindowEvent::HoveredFile(_) => todo!(),
            | event::WindowEvent::HoveredFileCancelled => todo!(),
            | event::WindowEvent::Ime(_) => todo!(),
            | event::WindowEvent::PinchGesture { .. } => todo!(),
            | event::WindowEvent::PanGesture { .. } => todo!(),
            | event::WindowEvent::DoubleTapGesture { .. } => todo!(),
            | event::WindowEvent::RotationGesture { .. } => todo!(),
            | event::WindowEvent::TouchpadPressure { .. } => todo!(),
            | event::WindowEvent::AxisMotion { .. } => todo!(),
            | event::WindowEvent::ScaleFactorChanged { .. } => todo!(),
            | event::WindowEvent::ThemeChanged(_) => todo!(),

            | event::WindowEvent::ModifiersChanged(_) => {}
            | event::WindowEvent::Touch(_) => {}
            | event::WindowEvent::MouseWheel { .. } => {}
            | event::WindowEvent::Occluded(_) => {}
            | event::WindowEvent::Moved(_) => {}
            | event::WindowEvent::CursorLeft { .. } => {}
            | event::WindowEvent::CursorMoved { .. } => {}
            | event::WindowEvent::CursorEntered { .. } => {}
            | event::WindowEvent::Focused(_) => {
                log::info!("Window focused");
            }
            | event::WindowEvent::Destroyed => {
                log::warn!("Window destroyed");
                event_loop.exit();
            }
            | event::WindowEvent::CloseRequested => {
                log::warn!("Close requested");
                event_loop.exit();
            }
            | event::WindowEvent::Resized(physical_size) => {
                log::info!("Resize requested: {:?}", physical_size);
                state.config_changed(physical_size.width, physical_size.height);
            }
            | event::WindowEvent::MouseInput { state: ele_state, button, .. } => {
                log::debug!("Mouse pressed: {:?}", button);
                match ele_state {
                    | event::ElementState::Pressed => {
                        let (left, right) = &mut state.input.mouse_pressed;
                        match button {
                            | event::MouseButton::Left => *left = true,
                            | event::MouseButton::Right => *right = true,
                            | event::MouseButton::Middle => todo!(),
                            | event::MouseButton::Back => todo!(),
                            | event::MouseButton::Forward => todo!(),
                            | event::MouseButton::Other(_) => todo!(),
                        }
                    }
                    | event::ElementState::Released => {
                        let (left, right) = &mut state.input.mouse_released;
                        match button {
                            | event::MouseButton::Left => *left = true,
                            | event::MouseButton::Right => *right = true,
                            | event::MouseButton::Middle => todo!(),
                            | event::MouseButton::Back => todo!(),
                            | event::MouseButton::Forward => todo!(),
                            | event::MouseButton::Other(_) => todo!(),
                        }
                    }
                }
            }
            | event::WindowEvent::KeyboardInput { event, .. } => {
                log::debug!("Keyboard input: {:?}", event);
                let keycode = match event.physical_key {
                    | keyboard::PhysicalKey::Code(key_code) => key_code,
                    | keyboard::PhysicalKey::Unidentified(native_key_code) => {
                        log::error!("Unidentified keycode: {:?}", native_key_code);
                        return;
                    }
                };

                let name = input::keycode_name(&keycode);
                match event.state {
                    | event::ElementState::Pressed => {
                        state.input.key_pressed.insert(name);
                        state.input.key_released.remove(name);
                    }
                    | event::ElementState::Released => {
                        state.input.key_pressed.remove(name);
                        state.input.key_released.insert(name);
                    }
                }
            }
            | event::WindowEvent::RedrawRequested => {
                if state.input.request_quit {
                    log::warn!("Quit requested");
                    event_loop.exit();
                    return;
                }

                match state.input.request_grab {
                    | true => {
                        state.window.set_cursor_grab(window::CursorGrabMode::Confined).unwrap();
                        state.window.set_cursor_visible(false);
                    }
                    | false => {
                        state.window.set_cursor_grab(window::CursorGrabMode::None).unwrap();
                        state.window.set_cursor_visible(true);
                    }
                }

                if let Err(err) = state.update() {
                    log::error!("Update error: {}", err);
                    event_loop.exit();
                }

                if let Err(err) = state.render() {
                    log::error!("Render error: {}", err);
                    event_loop.exit();
                }
            }
        }
    }
}
