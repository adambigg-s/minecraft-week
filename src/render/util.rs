use crate::render::mesh;
use crate::render::resource;
use crate::render::{self};

pub fn mesh<Vertex, Index>(
     context: &render::GfxContext,
     vertices: &[Vertex],
     indices: &[Index],
) -> mesh::GfxMesh
where
     Vertex: render::GfxVertex,
     Index: Into<u32> + Copy,
{
     mesh::GfxMesh::new(context, vertices, indices)
}

pub fn uniform<Uniform>(context: &render::GfxContext, label: &str) -> resource::GfxResource
where
     Uniform: bytemuck::Pod,
{
     resource::GfxResource::Uniform(resource::GfxUniform::new::<Uniform>(context, label))
}

pub fn texture(context: &render::GfxContext, path: &str, label: &str)
-> anyhow::Result<resource::GfxResource>
{
     Ok(resource::GfxResource::Texture(resource::GfxTexture::new(context, path, label)?))
}

pub fn texture_image(
     context: &render::GfxContext,
     image: &image::RgbaImage,
     label: &str,
) -> resource::GfxResource
{
     resource::GfxResource::Texture(resource::GfxTexture::new_image(context, image, label))
}

pub fn sampler(context: &render::GfxContext, label: &str) -> resource::GfxResource
{
     resource::GfxResource::Sampler(context.device.create_sampler(&wgpu::SamplerDescriptor {
          label: Some(&format!("{} sampler", label)),
          address_mode_u: wgpu::AddressMode::Repeat,
          address_mode_v: wgpu::AddressMode::Repeat,
          address_mode_w: wgpu::AddressMode::Repeat,
          mag_filter: wgpu::FilterMode::Nearest,
          min_filter: wgpu::FilterMode::Nearest,
          mipmap_filter: wgpu::MipmapFilterMode::Nearest,
          ..Default::default()
     }))
}
