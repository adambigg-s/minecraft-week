use std::mem;

use wgpu::vertex_attr_array;

use crate::engine::rectilinear;
use crate::render::mesh;
use crate::render::{self};
use crate::visual::atlas;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct SkyboxVertex
{
     pub pos: glam::Vec3,
     pub tex: glam::Vec2,
}

impl render::GfxVertex for SkyboxVertex
{
     fn descriptor() -> wgpu::VertexBufferLayout<'static>
     {
          const ATTRIBS: &[wgpu::VertexAttribute] = &vertex_attr_array![
              0 => Float32x3,
              1 => Float32x2,
          ];

          wgpu::VertexBufferLayout {
               array_stride: mem::size_of::<Self>() as u64,
               step_mode: wgpu::VertexStepMode::Vertex,
               attributes: ATTRIBS,
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct Skybox
{
     pub texture: atlas::TextureAtlas,
     pub mesh: rectilinear::RectilinearMesh,
}

impl Skybox
{
     pub fn new(directory: &str, tile_size: u32, absolute_size: f32) -> anyhow::Result<Self>
     {
          let texture = atlas::TextureAtlas::new(directory, tile_size)?;
          let mut mesh = rectilinear::RectilinearMesh::unit_cube();
          mesh.shift(glam::Vec3::splat(-0.5));
          mesh.scale(glam::Vec3::splat(absolute_size));

          Ok(Self {
               texture,
               mesh,
          })
     }

     pub fn create_gfx_mesh(&mut self, context: &render::GfxContext) -> mesh::GfxMesh
     {
          let mut vertices = Vec::new();
          (0 .. self.mesh.size).for_each(|index| {
               let rectilinear::RectilinearMeshSlice {
                    face,
                    pos,
                    uvs,
                    ..
               } = self.mesh.quad_slice(index);

               self.texture.conform_uvs(uvs, "skybox", face);
               (0 .. 4).for_each(|vertex| {
                    vertices.push(SkyboxVertex {
                         pos: pos[vertex],
                         tex: uvs[vertex],
                    });
               });
          });
          let indices = &self.mesh.index;

          mesh::GfxMesh::new(context, &vertices, indices)
     }
}
