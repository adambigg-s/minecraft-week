use std::sync;

use winit::application as winit_application;
use winit::dpi;
use winit::event;
use winit::event_loop;
use winit::keyboard;
use winit::window;

use crate::application::input;
use crate::application::state;
use crate::application::{self};

#[derive(bon::Builder, Debug, Default)]
pub struct ApplicationRunner<Inner>
{
     pub inner: Option<state::State<Inner>>,
}

impl<Inner> ApplicationRunner<Inner>
{
     pub fn new() -> Self
     {
          Self {
               inner: None,
          }
     }
}

impl<Inner> winit_application::ApplicationHandler for ApplicationRunner<Inner>
where
     Inner: application::Application,
{
     fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop)
     {
          log::warn!("Application resume requested");

          if self.inner.is_some()
          {
               log::error!("False resume");
               return;
          }

          let config = Inner::config();
          let window_attrs = window::WindowAttributes::default()
               .with_window_level(window::WindowLevel::Normal)
               .with_position(dpi::PhysicalPosition::new(1100, 100))
               .with_inner_size(dpi::PhysicalSize::new(config.width, config.height))
               .with_title(config.title);
          let window = sync::Arc::new(event_loop.create_window(window_attrs).unwrap());

          self.inner = Some(pollster::block_on(state::State::new(window)).unwrap());
     }

     fn device_event(
          &mut self,
          _: &event_loop::ActiveEventLoop,
          _: event::DeviceId,
          event: event::DeviceEvent,
     )
     {
          let state = match &mut self.inner
          {
               | Some(state) => state,
               | None =>
               {
                    log::error!("False device event");
                    return;
               }
          };

          if let event::DeviceEvent::MouseMotion {
               delta: (dx, dy),
          } = event
          {
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
     )
     {
          let state = match &mut self.inner
          {
               | Some(state) => state,
               | None =>
               {
                    log::error!("False window event");
                    return;
               }
          };

          match event
          {
               | event::WindowEvent::ActivationTokenDone {
                    ..
               } => todo!(),
               | event::WindowEvent::DroppedFile(_) => todo!(),
               | event::WindowEvent::HoveredFile(_) => todo!(),
               | event::WindowEvent::HoveredFileCancelled => todo!(),
               | event::WindowEvent::Ime(_) => todo!(),
               | event::WindowEvent::PinchGesture {
                    ..
               } => todo!(),
               | event::WindowEvent::PanGesture {
                    ..
               } => todo!(),
               | event::WindowEvent::DoubleTapGesture {
                    ..
               } => todo!(),
               | event::WindowEvent::RotationGesture {
                    ..
               } => todo!(),
               | event::WindowEvent::TouchpadPressure {
                    ..
               } => todo!(),
               | event::WindowEvent::AxisMotion {
                    ..
               } => todo!(),
               | event::WindowEvent::ScaleFactorChanged {
                    ..
               } => todo!(),
               | event::WindowEvent::ThemeChanged(_) => todo!(),

               | event::WindowEvent::ModifiersChanged(_) =>
               {}
               | event::WindowEvent::Touch(_) =>
               {}
               | event::WindowEvent::MouseWheel {
                    ..
               } =>
               {}
               | event::WindowEvent::Occluded(_) =>
               {}
               | event::WindowEvent::Moved(_) =>
               {}
               | event::WindowEvent::CursorLeft {
                    ..
               } =>
               {}
               | event::WindowEvent::CursorMoved {
                    ..
               } =>
               {}
               | event::WindowEvent::CursorEntered {
                    ..
               } =>
               {}

               | event::WindowEvent::Focused(_) =>
               {
                    log::info!("Window focused");
               }
               | event::WindowEvent::Destroyed =>
               {
                    log::warn!("Window destroyed");
                    event_loop.exit();
               }
               | event::WindowEvent::CloseRequested =>
               {
                    log::warn!("Close requested");
                    event_loop.exit();
               }
               | event::WindowEvent::Resized(physical_size) =>
               {
                    log::debug!("Resize requested: {:?}", physical_size);
                    state.config_changed(physical_size.width, physical_size.height).unwrap();
               }
               | event::WindowEvent::MouseInput {
                    state: ele_state,
                    button,
                    ..
               } =>
               {
                    let (left_press, right_press) = &mut state.input.mouse_pressed;
                    let (left_release, right_release) = &mut state.input.mouse_released;
                    match ele_state
                    {
                         | event::ElementState::Pressed =>
                         {
                              match button
                              {
                                   | event::MouseButton::Left =>
                                   {
                                        *left_press = true;
                                        *left_release = false
                                   }
                                   | event::MouseButton::Right =>
                                   {
                                        *right_press = true;
                                        *right_release = false
                                   }
                                   | event::MouseButton::Middle => todo!(),
                                   | event::MouseButton::Back => todo!(),
                                   | event::MouseButton::Forward => todo!(),
                                   | event::MouseButton::Other(_) => todo!(),
                              }
                         }
                         | event::ElementState::Released =>
                         {
                              match button
                              {
                                   | event::MouseButton::Left =>
                                   {
                                        *left_release = true;
                                        *left_press = false;
                                   }
                                   | event::MouseButton::Right =>
                                   {
                                        *right_release = true;
                                        *right_press = false;
                                   }
                                   | event::MouseButton::Middle => todo!(),
                                   | event::MouseButton::Back => todo!(),
                                   | event::MouseButton::Forward => todo!(),
                                   | event::MouseButton::Other(_) => todo!(),
                              }
                         }
                    }
               }
               | event::WindowEvent::KeyboardInput {
                    event, ..
               } =>
               {
                    let keyboard::PhysicalKey::Code(keycode) = event.physical_key
                    else
                    {
                         return;
                    };

                    let name = input::keycode_name(&keycode);
                    match event.state
                    {
                         | event::ElementState::Pressed =>
                         {
                              state.input.key_pressed.insert(name);
                              state.input.key_released.remove(name);
                         }
                         | event::ElementState::Released =>
                         {
                              state.input.key_pressed.remove(name);
                              state.input.key_released.insert(name);
                         }
                    }
               }
               | event::WindowEvent::RedrawRequested =>
               {
                    if state.input.request_quit
                    {
                         log::warn!("Quit requested");
                         event_loop.exit();
                         return;
                    }

                    match state.input.request_grab
                    {
                         | input::MouseMode::None =>
                         {
                              state.window.set_cursor_grab(window::CursorGrabMode::None).unwrap();
                              state.window.set_cursor_visible(true);
                              state.input.consume_mouse_delta();
                         }
                         | input::MouseMode::Grab =>
                         {
                              state.window.set_cursor_grab(window::CursorGrabMode::Confined).unwrap();
                              state.window.set_cursor_visible(false);
                         }
                         | input::MouseMode::Free =>
                         {
                              state.window.set_cursor_grab(window::CursorGrabMode::None).unwrap();
                              state.window.set_cursor_visible(true);
                         }
                    }

                    if let Err(err) = state.update()
                    {
                         log::error!("Update error: {}", err);
                         event_loop.exit();
                    }

                    if let Err(err) = state.render()
                    {
                         log::error!("Render error: {}", err);
                         event_loop.exit();
                    }
               }
          };
     }
}
