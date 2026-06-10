use std::mem;

use crate::render::{self, GfxContext};

#[derive(Debug)]
pub enum GfxBindingLayout {
    Uniform,
    Texture,
    Sampler,
}

#[derive(Debug)]
pub enum GfxBindingData<'d> {
    Uniform(&'d GfxUniform),
    Texture(&'d GfxTexture),
    Sampler(&'d wgpu::Sampler),
}

#[derive(Debug)]
pub enum GfxResource {
    Uniform(GfxUniform),
    Texture(GfxTexture),
    Sampler(wgpu::Sampler),
}

#[derive(bon::Builder, Debug)]
pub struct GfxUniform {
    pub buffer: wgpu::Buffer,
}

impl GfxUniform {
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

        Self { buffer }
    }

    pub fn write<Uniform>(&self, context: &GfxContext, data: &Uniform)
    where
        Uniform: bytemuck::Pod,
    {
        context.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }
}

#[derive(bon::Builder, Debug)]
pub struct GfxTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl GfxTexture {
    pub fn new(context: &GfxContext, path: &str, label: &str) -> anyhow::Result<Self> {
        let image = image::open(path)?.flipv();
        let rgba = image.to_rgba8();

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
            format: context.config.format,
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
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture.width()),
                rows_per_image: Some(texture.height()),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self { texture, view })
    }
}
