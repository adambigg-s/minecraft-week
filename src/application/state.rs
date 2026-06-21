use std::sync;

use winit::window;

use crate::application::input;
use crate::application::{self};
use crate::render;

#[derive(bon::Builder, Debug)]
pub struct State<Inner>
{
     pub window: sync::Arc<window::Window>,
     pub gfx_context: render::GfxContext,
     pub gfx_render: render::GfxRenderer,
     pub input: input::Input,
     pub inner_state: Inner,
}

impl<Inner> State<Inner>
where
     Inner: application::Application,
{
     pub async fn new(window: sync::Arc<window::Window>) -> anyhow::Result<Self>
     {
          let mut gfx_context = pollster::block_on(render::GfxContext::new(sync::Arc::clone(&window)))?;

          let mut gfx_render = render::GfxRenderer::new(&gfx_context)?;

          let input = input::Input::new();

          let inner_state = Inner::setup(&mut gfx_context, &mut gfx_render)?;

          Ok(Self {
               window,
               gfx_context,
               gfx_render,
               input,
               inner_state,
          })
     }

     pub fn config_changed(&mut self, width: u32, height: u32) -> anyhow::Result<()>
     {
          self.gfx_context.config_changed(width, height);
          self.gfx_render.config_changed(&self.gfx_context)?;
          Ok(())
     }

     pub fn update(&mut self) -> anyhow::Result<()>
     {
          self.inner_state.physics_frame(&mut self.input, &self.gfx_context, &self.gfx_render);
          Ok(())
     }

     pub fn screenshot(&self) -> anyhow::Result<()>
     {
          Ok(())
     }

     pub fn render(&mut self) -> anyhow::Result<()>
     {
          self.window.request_redraw();
          self.inner_state.gfx_frame(&self.input, &mut self.gfx_context, &mut self.gfx_render);

          let output = match self.gfx_context.surface.get_current_texture()
          {
               | wgpu::CurrentSurfaceTexture::Timeout
               | wgpu::CurrentSurfaceTexture::Occluded
               | wgpu::CurrentSurfaceTexture::Outdated
               | wgpu::CurrentSurfaceTexture::Validation =>
               {
                    return Ok(());
               }
               | wgpu::CurrentSurfaceTexture::Lost =>
               {
                    anyhow::bail!("Device lost");
               }
               | wgpu::CurrentSurfaceTexture::Success(surface_texture)
               | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
          };

          let mut encoder = self.gfx_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
               label: Some("Command encoder"),
          });

          {
               let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                         view: &output.texture.create_view(&wgpu::TextureViewDescriptor::default()),
                         depth_slice: None,
                         resolve_target: None,
                         ops: wgpu::Operations {
                              load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                              store: wgpu::StoreOp::Store,
                         },
                    })],
                    depth_stencil_attachment: match &self.gfx_render.depth_texture
                    {
                         | Some(depth_texture) =>
                         {
                              Some(wgpu::RenderPassDepthStencilAttachment {
                                   view: &depth_texture.view,
                                   depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: wgpu::StoreOp::Store,
                                   }),
                                   stencil_ops: None,
                              })
                         }
                         | None => None,
                    },
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
