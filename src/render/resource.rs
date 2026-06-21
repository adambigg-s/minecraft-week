use std::mem;

use crate::render::GfxContext;
use crate::render::{self};

#[derive(Debug)]
pub enum GfxBindingLayout
{
     Uniform,
     Texture,
     Sampler,
}

impl GfxBindingLayout
{
     pub fn get_bind_group_layout(&self, index: u32) -> wgpu::BindGroupLayoutEntry
     {
          wgpu::BindGroupLayoutEntry {
               binding: index,
               visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
               ty: match self
               {
                    | GfxBindingLayout::Uniform =>
                    {
                         wgpu::BindingType::Buffer {
                              ty: wgpu::BufferBindingType::Uniform,
                              has_dynamic_offset: false,
                              min_binding_size: None,
                         }
                    }
                    | GfxBindingLayout::Texture =>
                    {
                         wgpu::BindingType::Texture {
                              sample_type: wgpu::TextureSampleType::Float {
                                   filterable: false,
                              },
                              view_dimension: wgpu::TextureViewDimension::D2,
                              multisampled: false,
                         }
                    }
                    | GfxBindingLayout::Sampler =>
                    {
                         wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering)
                    }
               },
               count: None,
          }
     }
}

#[derive(Debug)]
pub enum GfxResource
{
     Uniform(GfxUniform),
     Texture(GfxTexture),
     Sampler(wgpu::Sampler),
}

impl GfxResource
{
     pub fn get_bind_group<'d>(&'d self, index: u32) -> wgpu::BindGroupEntry<'d>
     {
          wgpu::BindGroupEntry {
               binding: index,
               resource: match self
               {
                    | GfxResource::Uniform(gfx_uniform) => gfx_uniform.buffer.as_entire_binding(),
                    | GfxResource::Texture(gfx_texture) =>
                    {
                         wgpu::BindingResource::TextureView(&gfx_texture.view)
                    }
                    | GfxResource::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
               },
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct GfxUniform
{
     pub buffer: wgpu::Buffer,
}

impl GfxUniform
{
     pub fn new<Uniform>(context: &render::GfxContext, label: &str) -> Self
     where
          Uniform: bytemuck::Pod,
     {
          let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
               label: Some(&format!("{} uniform", label)),
               size: mem::size_of::<Uniform>() as u64,
               usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
               mapped_at_creation: false,
          });

          Self {
               buffer,
          }
     }

     pub fn write<Uniform>(&self, context: &GfxContext, data: &Uniform)
     where
          Uniform: bytemuck::Pod,
     {
          context.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
     }
}

#[derive(bon::Builder, Debug)]
pub struct GfxTexture
{
     pub texture: wgpu::Texture,
     pub view: wgpu::TextureView,
}

impl GfxTexture
{
     pub fn new(context: &GfxContext, path: &str, label: &str) -> anyhow::Result<Self>
     {
          let image = image::open(path)?.flipv();
          let rgba = image.to_rgba8();

          Ok(Self::new_image(context, &rgba, label))
     }

     pub fn new_image(context: &GfxContext, image: &image::RgbaImage, label: &str) -> Self
     {
          let size = wgpu::Extent3d {
               width: image.width(),
               height: image.height(),
               depth_or_array_layers: 1,
          };
          let texture = context.device.create_texture(&wgpu::TextureDescriptor {
               label: Some(&format!("{} texture", label)),
               size,
               mip_level_count: 1,
               sample_count: 1,
               dimension: wgpu::TextureDimension::D2,
               format: wgpu::TextureFormat::Rgba8UnormSrgb,
               usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
               view_formats: &[],
          });
          context.queue.write_texture(
               wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
               },
               image,
               wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * texture.width()),
                    rows_per_image: Some(texture.height()),
               },
               size,
          );
          let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

          Self {
               texture,
               view,
          }
     }

     pub fn new_depth(context: &GfxContext, label: &str) -> anyhow::Result<Self>
     {
          let texture = context.device.create_texture(&wgpu::TextureDescriptor {
               label: Some(&format!("{} depth texture", label)),
               size: wgpu::Extent3d {
                    width: context.config.width,
                    height: context.config.height,
                    depth_or_array_layers: 1,
               },
               mip_level_count: 1,
               sample_count: 1,
               dimension: wgpu::TextureDimension::D2,
               format: wgpu::TextureFormat::Depth32Float,
               usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
               view_formats: &[],
          });
          let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

          Ok(Self {
               texture,
               view,
          })
     }
}
