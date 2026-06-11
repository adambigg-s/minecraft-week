use crate::render::{self, GfxContext, mesh, resources};

pub fn mesh<Vertex, Index>(
    context: &render::GfxContext,
    vertices: &[Vertex],
    indices: &[Index],
) -> mesh::GfxMesh
where
    Vertex: render::GfxVertex,
    Index: Into<u16> + Copy,
{
    mesh::GfxMesh::new(context, vertices, indices)
}

pub fn uniform<Uniform>(context: &render::GfxContext, label: &str) -> resources::GfxResource
where
    Uniform: bytemuck::Pod,
{
    resources::GfxResource::Uniform(resources::GfxUniform::new::<Uniform>(context, label))
}

pub fn texture(
    context: &render::GfxContext,
    path: &str,
    label: &str,
) -> anyhow::Result<resources::GfxResource> {
    Ok(resources::GfxResource::Texture(resources::GfxTexture::new(context, path, label)?))
}

pub fn sampler(context: &GfxContext, label: &str) -> resources::GfxResource {
    resources::GfxResource::Sampler(context.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("{} sampler", label)),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    }))
}
