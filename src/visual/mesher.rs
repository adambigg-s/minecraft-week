use std::mem;

use wgpu::vertex_attr_array;

use crate::engine::rectilinear;
use crate::render::{self};
use crate::visual::atlas;
use crate::visual::light;
use crate::world::block;
use crate::world::{self};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MeshType
{
     #[default]
     Opaque,
     Transparent,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct TerrainVertex
{
     pub pos: glam::Vec3,
     pub nor: glam::Vec3,
     pub tex: glam::Vec2,
     pub fil: f32,
     pub bil: f32,
     pub ao: f32,
}

impl render::GfxVertex for TerrainVertex
{
     fn descriptor() -> wgpu::VertexBufferLayout<'static>
     {
          const ATTRIBS: &[wgpu::VertexAttribute] = &vertex_attr_array![
              0 => Float32x3,
              1 => Float32x3,
              2 => Float32x2,
              3 => Float32,
              4 => Float32,
              5 => Float32,
          ];

          wgpu::VertexBufferLayout {
               array_stride: mem::size_of::<Self>() as u64,
               step_mode: wgpu::VertexStepMode::Vertex,
               attributes: ATTRIBS,
          }
     }
}

#[derive(bon::Builder, Debug, Default)]
pub struct TerrainQuad
{
     pub quad: rectilinear::Quad,
     pub block: block::Block,
     pub ambient_occlusion: [f32; 4],
     pub face_illumination: [f32; 4],
     pub block_illumination: [f32; 4],
}

impl TerrainQuad
{
     pub fn indices(&self, start: u32) -> [u32; 6]
     {
          let out;

          let ao0 = self.ambient_occlusion[0] + self.ambient_occlusion[3];
          let ao1 = self.ambient_occlusion[1] + self.ambient_occlusion[2];
          let fl0 = self.face_illumination[0] + self.face_illumination[3];
          let fl1 = self.face_illumination[1] + self.face_illumination[2];

          if ao0 < ao1
          {
               out = [start, start + 2, start + 1, start + 1, start + 2, start + 3];
          }
          else if ao0 > ao1
          {
               out = [start, start + 2, start + 3, start, start + 3, start + 1];
          }
          else
          {
               if fl0 < fl1
               {
                    out = [start, start + 2, start + 3, start, start + 3, start + 1];
               }
               else
               {
                    out = [start, start + 2, start + 1, start + 1, start + 2, start + 3];
               }
          }

          out
     }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecoratorBillboard
{
     DiagPosFront,
     DiagPosBack,
     DiagNegFront,
     DiagNegBack,
}

impl DecoratorBillboard
{
     const DIAGONALS: [DecoratorBillboard; 4] = [
          DecoratorBillboard::DiagPosFront,
          DecoratorBillboard::DiagPosBack,
          DecoratorBillboard::DiagNegFront,
          DecoratorBillboard::DiagNegBack,
     ];

     pub fn diagonals() -> [Self; 4]
     {
          Self::DIAGONALS
     }

     pub fn normal(&self) -> glam::Vec3
     {
          match self
          {
               | DecoratorBillboard::DiagPosFront => glam::ivec3(1, 0, -1).as_vec3().normalize(),
               | DecoratorBillboard::DiagPosBack => glam::ivec3(1, 0, 1).as_vec3().normalize(),
               | DecoratorBillboard::DiagNegFront => glam::ivec3(-1, 0, -1).as_vec3().normalize(),
               | DecoratorBillboard::DiagNegBack => glam::ivec3(-1, 0, 1).as_vec3().normalize(),
          }
     }

     pub fn corners(&self) -> [(glam::IVec3, glam::IVec2); 4]
     {
          use glam::ivec2;
          use glam::ivec3;

          match self
          {
               | DecoratorBillboard::DiagPosFront =>
               {
                    [
                         (ivec3(0, 1, 0), ivec2(0, 0)),
                         (ivec3(0, 0, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 1), ivec2(1, 0)),
                         (ivec3(1, 0, 1), ivec2(1, 1)),
                    ]
               }
               | DecoratorBillboard::DiagPosBack =>
               {
                    [
                         (ivec3(1, 1, 1), ivec2(0, 0)),
                         (ivec3(1, 0, 1), ivec2(0, 1)),
                         (ivec3(0, 1, 0), ivec2(1, 0)),
                         (ivec3(0, 0, 0), ivec2(1, 1)),
                    ]
               }
               | DecoratorBillboard::DiagNegFront =>
               {
                    [
                         (ivec3(1, 1, 0), ivec2(0, 0)),
                         (ivec3(1, 0, 0), ivec2(0, 1)),
                         (ivec3(0, 1, 1), ivec2(1, 0)),
                         (ivec3(0, 0, 1), ivec2(1, 1)),
                    ]
               }
               | DecoratorBillboard::DiagNegBack =>
               {
                    [
                         (ivec3(0, 1, 1), ivec2(0, 0)),
                         (ivec3(0, 0, 1), ivec2(0, 1)),
                         (ivec3(1, 1, 0), ivec2(1, 0)),
                         (ivec3(1, 0, 0), ivec2(1, 1)),
                    ]
               }
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct TerrainDecorator
{
     pub position: glam::IVec3,
     pub decorator: DecoratorBillboard,
}

impl TerrainDecorator
{
     pub fn positions(&self) -> [glam::Vec3; 4]
     {
          self.decorator.corners().map(|(offset, _)| (self.position + offset).as_vec3())
     }

     pub fn texture_uvs(&self) -> [glam::Vec2; 4]
     {
          self.decorator.corners().map(|(_, uv)| uv.as_vec2())
     }

     pub fn normals(&self) -> [glam::Vec3; 4]
     {
          [self.decorator.normal(); 4]
     }

     pub fn indices(&self, start: u32) -> [u32; 6]
     {
          [start, start + 2, start + 1, start + 1, start + 2, start + 3]
     }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkRawMesh
{
     pub vertices: Vec<TerrainVertex>,
     pub indices: Vec<u32>,
     pub offset: glam::IVec3,
     pub mesh_type: MeshType,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkMesher<'c>
{
     pub view: &'c world::ChunkView,
     pub atlas: &'c atlas::TextureAtlas,
}

impl<'c> ChunkMesher<'c>
{
     pub fn raw_opaque_mesh(&self) -> ChunkRawMesh
     {
          use block::Visibility::*;

          let chunk = &self.view.chunk;
          let mut vertices = Vec::new();
          let mut indices = Vec::new();
          for idx in chunk.indices()
          {
               let coord = glam::IVec3::from_array(chunk.delinearize(idx).map(|val| val as i32));
               let world_coord = chunk.world_position() + coord;
               let block = *chunk.get(coord);
               let light = self.view.get_light(coord).luminosity();

               if block.visibility() == Invisible
               {
                    continue;
               }

               match block.mesh_style()
               {
                    | block::EmittedMesh::RectilinearFull =>
                    {
                         for face in rectilinear::Face::cardinals()
                         {
                              let offset = face.neighbor_offset();
                              let neighbor_coord = coord + offset;
                              let neighbor_block = self.view.get_block(neighbor_coord);

                              let emit = match (block.visibility(), neighbor_block.visibility())
                              {
                                   | (Opaque, PartialOpaque)
                                   | (Opaque, Transparent)
                                   | (Opaque, Invisible)
                                   | (PartialOpaque, PartialOpaque)
                                   | (PartialOpaque, Transparent)
                                   | (PartialOpaque, Invisible)
                                   | (Transparent, Invisible)
                                   | (Transparent, PartialOpaque) => true,
                                   | _ => false,
                              };
                              if !emit
                              {
                                   continue;
                              }

                              self.append_block_full_face(
                                   &mut vertices,
                                   &mut indices,
                                   coord,
                                   world_coord,
                                   block,
                                   light,
                                   face,
                              );
                         }
                    }
                    | block::EmittedMesh::Decorator =>
                    {
                         for decorator in DecoratorBillboard::diagonals()
                         {
                              self.append_decorator(
                                   &mut vertices,
                                   &mut indices,
                                   world_coord,
                                   coord,
                                   block,
                                   light,
                                   decorator,
                              );
                         }
                    }
                    | block::EmittedMesh::RectilinearPartial => todo!(),
               }
          }
          let offset = chunk.offset();
          let mesh_type = MeshType::Opaque;

          ChunkRawMesh {
               vertices,
               indices,
               offset,
               mesh_type,
          }
     }

     pub fn raw_transparent_mesh(&self) -> ChunkRawMesh
     {
          todo!()
     }

     fn map_ao(&self, coord: glam::IVec3, face: rectilinear::Face) -> [f32; 4]
     {
          let nor = face.neighbor_offset();
          let adj = coord + nor;

          face.corners().map(|(offset, _)| {
               let dir = offset * 2 - glam::IVec3::ONE;
               let tn_cand = dir * (glam::IVec3::ONE - nor.abs());

               let (tn, btn) = if nor.x != 0
               {
                    (glam::ivec3(0, tn_cand.y, 0), glam::ivec3(0, 0, tn_cand.z))
               }
               else if nor.y != 0
               {
                    (glam::ivec3(tn_cand.x, 0, 0), glam::ivec3(0, 0, tn_cand.z))
               }
               else
               {
                    (glam::ivec3(tn_cand.x, 0, 0), glam::ivec3(0, tn_cand.y, 0))
               };

               let side1 = *self.view.get_block(adj + tn).opacity() != 0;
               let side2 = *self.view.get_block(adj + btn).opacity() != 0;
               let corner = *self.view.get_block(adj + tn + btn).opacity() != 0;

               let occlusion = match (side1, side2)
               {
                    | (true, true) => 0,
                    | _ => 3 - (side1 as i32 + side2 as i32 + corner as i32),
               };

               (occlusion as f32 + 1.0) * 0.25
          })
     }

     fn map_face_lighting(&self, coord: glam::IVec3, face: rectilinear::Face) -> [f32; 4]
     {
          let nor = face.neighbor_offset();
          let adj = coord + nor;

          face.corners().map(|(offset, _)| {
               let dir = offset * 2 - glam::IVec3::ONE;
               let tn_cand = dir * (glam::IVec3::ONE - nor.abs());

               let (tn, btn) = if nor.x != 0
               {
                    (glam::ivec3(0, tn_cand.y, 0), glam::ivec3(0, 0, tn_cand.z))
               }
               else if nor.y != 0
               {
                    (glam::ivec3(tn_cand.x, 0, 0), glam::ivec3(0, 0, tn_cand.z))
               }
               else
               {
                    (glam::ivec3(tn_cand.x, 0, 0), glam::ivec3(0, tn_cand.y, 0))
               };

               let center_light = *self.view.get_light(adj) as f32;
               let side1_light = *self.view.get_light(adj + tn) as f32;
               let side2_light = *self.view.get_light(adj + btn) as f32;
               let mut corner_light = *self.view.get_light(adj + tn + btn) as f32;

               let side1 = *self.view.get_block(adj + tn).opacity() != 0;
               let side2 = *self.view.get_block(adj + btn).opacity() != 0;

               if side1 && side2
               {
                    corner_light = center_light;
               }

               (center_light + side1_light + side2_light + corner_light)
                    / (4.0 * *light::Light::max_light() as f32)
          })
     }

     #[allow(clippy::too_many_arguments)]
     fn append_block_full_face(
          &self,
          vertices: &mut Vec<TerrainVertex>,
          indices: &mut Vec<u32>,
          coord: glam::IVec3,
          world_coord: glam::IVec3,
          block: block::Block,
          block_light: f32,
          face: rectilinear::Face,
     )
     {
          let ao = self.map_ao(coord, face);
          let face_light = self.map_face_lighting(coord, face);

          let quad = TerrainQuad {
               quad: rectilinear::Quad {
                    position: world_coord,
                    face,
               },
               block,
               ambient_occlusion: ao,
               face_illumination: face_light,
               block_illumination: [block_light; 4],
          };
          let pos = quad.quad.positions();
          let nor = quad.quad.normals();
          let fil = quad.face_illumination;
          let bil = quad.block_illumination;
          let ao = quad.ambient_occlusion;
          let mut tex = quad.quad.texture_uvs();
          self.atlas.conform_uvs(&mut tex, block.name(), face);
          indices.extend_from_slice(&quad.indices(vertices.len() as u32));
          (0 .. 4).for_each(|vertex| {
               vertices.push(TerrainVertex {
                    pos: pos[vertex],
                    nor: nor[vertex],
                    tex: tex[vertex],
                    fil: fil[vertex],
                    bil: bil[vertex],
                    ao: ao[vertex],
               });
          });
     }

     #[allow(unused)]
     #[allow(clippy::too_many_arguments)]
     fn append_decorator(
          &self,
          vertices: &mut Vec<TerrainVertex>,
          indices: &mut Vec<u32>,
          world_coord: glam::IVec3,
          coord: glam::IVec3,
          block: block::Block,
          block_light: f32,
          decorator: DecoratorBillboard,
     )
     {
          let decorator = TerrainDecorator {
               position: world_coord,
               decorator,
          };
          let pos = decorator.positions();
          let nor = decorator.normals();
          let mut tex = decorator.texture_uvs();
          self.atlas.conform_uvs(&mut tex, block.name(), rectilinear::Face::Front);
          indices.extend_from_slice(&decorator.indices(vertices.len() as u32));
          (0 .. 4).for_each(|vertex| {
               vertices.push(TerrainVertex {
                    pos: pos[vertex],
                    nor: nor[vertex],
                    tex: tex[vertex],
                    fil: block_light,
                    bil: block_light,
                    ao: 1.0,
               });
          });
     }
}
