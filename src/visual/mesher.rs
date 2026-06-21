use std::mem;

use wgpu::vertex_attr_array;

use crate::render::{self};
use crate::visual::atlas;
use crate::world::block;
use crate::world::light;
use crate::world::{self};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Face
{
     Top,
     Bottom,
     Left,
     Right,
     Back,
     Front,
     DiagPosFront,
     DiagPosBack,
     DiagNegFront,
     DiagNegBack,
}

impl Face
{
     pub const CARDINALS: [Face; 6] = [
          Face::Top,
          Face::Bottom,
          Face::Left,
          Face::Right,
          Face::Front,
          Face::Back,
     ];
     pub const DIAGONALS: [Face; 4] = [
          Face::DiagPosFront,
          Face::DiagPosBack,
          Face::DiagNegFront,
          Face::DiagNegBack,
     ];

     pub fn neighbor_offset(&self) -> glam::IVec3
     {
          match self
          {
               | Face::Top => glam::ivec3(0, 1, 0),
               | Face::Bottom => glam::ivec3(0, -1, 0),
               | Face::Left => glam::ivec3(-1, 0, 0),
               | Face::Right => glam::ivec3(1, 0, 0),
               | Face::Back => glam::ivec3(0, 0, 1),
               | Face::Front => glam::ivec3(0, 0, -1),
               | _ => glam::IVec3::ZERO,
          }
     }

     pub fn normal(&self) -> glam::Vec3
     {
          match self
          {
               | Face::Top => glam::Vec3::Y,
               | Face::Bottom => glam::Vec3::NEG_Y,
               | Face::Left => glam::Vec3::X,
               | Face::Right => glam::Vec3::NEG_X,
               | Face::Back => glam::Vec3::Z,
               | Face::Front => glam::Vec3::NEG_Z,
               | Face::DiagPosFront => glam::ivec3(1, 0, -1).as_vec3().normalize(),
               | Face::DiagPosBack => glam::ivec3(1, 0, 1).as_vec3().normalize(),
               | Face::DiagNegFront => glam::ivec3(-1, 0, -1).as_vec3().normalize(),
               | Face::DiagNegBack => glam::ivec3(-1, 0, 1).as_vec3().normalize(),
          }
     }

     pub fn corners(&self) -> [(glam::IVec3, glam::IVec2); 4]
     {
          use glam::ivec2;
          use glam::ivec3;

          match self
          {
               | Face::Top =>
               {
                    [
                         (ivec3(0, 1, 1), ivec2(0, 0)),
                         (ivec3(0, 1, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 1), ivec2(1, 0)),
                         (ivec3(1, 1, 0), ivec2(1, 1)),
                    ]
               }
               | Face::Bottom =>
               {
                    [
                         (ivec3(0, 0, 0), ivec2(0, 0)),
                         (ivec3(0, 0, 1), ivec2(0, 1)),
                         (ivec3(1, 0, 0), ivec2(1, 0)),
                         (ivec3(1, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::Left =>
               {
                    [
                         (ivec3(0, 1, 1), ivec2(0, 0)),
                         (ivec3(0, 0, 1), ivec2(0, 1)),
                         (ivec3(0, 1, 0), ivec2(1, 0)),
                         (ivec3(0, 0, 0), ivec2(1, 1)),
                    ]
               }
               | Face::Right =>
               {
                    [
                         (ivec3(1, 1, 0), ivec2(0, 0)),
                         (ivec3(1, 0, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 1), ivec2(1, 0)),
                         (ivec3(1, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::Back =>
               {
                    [
                         (ivec3(1, 1, 1), ivec2(0, 0)),
                         (ivec3(1, 0, 1), ivec2(0, 1)),
                         (ivec3(0, 1, 1), ivec2(1, 0)),
                         (ivec3(0, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::Front =>
               {
                    [
                         (ivec3(0, 1, 0), ivec2(0, 0)),
                         (ivec3(0, 0, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 0), ivec2(1, 0)),
                         (ivec3(1, 0, 0), ivec2(1, 1)),
                    ]
               }
               | Face::DiagPosFront =>
               {
                    [
                         (ivec3(0, 1, 0), ivec2(0, 0)),
                         (ivec3(0, 0, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 1), ivec2(1, 0)),
                         (ivec3(1, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::DiagPosBack =>
               {
                    [
                         (ivec3(1, 1, 1), ivec2(0, 0)),
                         (ivec3(1, 0, 1), ivec2(0, 1)),
                         (ivec3(0, 1, 0), ivec2(1, 0)),
                         (ivec3(0, 0, 0), ivec2(1, 1)),
                    ]
               }
               | Face::DiagNegFront =>
               {
                    [
                         (ivec3(1, 1, 0), ivec2(0, 0)),
                         (ivec3(1, 0, 0), ivec2(0, 1)),
                         (ivec3(0, 1, 1), ivec2(1, 0)),
                         (ivec3(0, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::DiagNegBack =>
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
pub struct Quad
{
     pub position: glam::IVec3,
     pub face: Face,
}

impl Quad
{
     pub fn cube() -> [Self; 6]
     {
          Face::CARDINALS.map(|face| Self { position: glam::IVec3::ZERO, face })
     }

     pub fn positions(&self) -> [glam::Vec3; 4]
     {
          self.face.corners().map(|(offset, _)| (self.position + offset).as_vec3())
     }

     pub fn texture_uvs(&self) -> [glam::Vec2; 4]
     {
          self.face.corners().map(|(_, uv)| uv.as_vec2())
     }

     pub fn normals(&self) -> [glam::Vec3; 4]
     {
          [self.face.normal(); 4]
     }

     pub fn indices(&self, start: u32) -> [u32; 6]
     {
          [start, start + 2, start + 1, start + 1, start + 2, start + 3]
     }
}

#[derive(bon::Builder, Debug)]
pub struct MeshQuad
{
     pub quad: Quad,
     pub ao: [f32; 4],
     pub lum: [f32; 4],
}

impl MeshQuad
{
     pub fn cube() -> [Self; 6]
     {
          Quad::cube().map(|quad| Self { quad, ao: [1.0; 4], lum: [1.0; 4] })
     }

     pub fn indices(&self, start: u32) -> [u32; 6]
     {
          if self.ao[0] + self.ao[3] < self.ao[1] + self.ao[2]
          {
               [start, start + 2, start + 1, start + 1, start + 2, start + 3]
          }
          else
          {
               [start, start + 2, start + 3, start, start + 3, start + 1]
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct RectilinearMeshSlice<'r>
{
     pub face: Face,
     pub integer_position: glam::IVec3,
     pub pos: &'r mut [glam::Vec3],
     pub nor: &'r mut [glam::Vec3],
     pub uvs: &'r mut [glam::Vec2],
     pub lum: &'r mut [f32],
     pub aos: &'r mut [f32],
}

#[derive(bon::Builder, Debug, Default)]
pub struct RectilinearMesh
{
     pub pos: Vec<glam::Vec3>,
     pub nor: Vec<glam::Vec3>,
     pub uvs: Vec<glam::Vec2>,
     pub lum: Vec<f32>,
     pub aos: Vec<f32>,
     pub index: Vec<u32>,
     pub integer_pos: Vec<glam::IVec3>,
     pub face: Vec<Face>,
     pub size: usize,
}

impl RectilinearMesh
{
     pub fn from_quads(quads: &[MeshQuad]) -> Self
     {
          let mut out = Self { size: quads.len(), ..Default::default() };
          quads.iter().for_each(|quad| {
               let len = out.pos.len();
               out.pos.extend_from_slice(&quad.quad.positions());
               out.nor.extend_from_slice(&quad.quad.normals());
               out.lum.extend_from_slice(&quad.lum);
               out.uvs.extend_from_slice(&quad.quad.texture_uvs());
               out.aos.extend_from_slice(&quad.ao);
               out.index.extend_from_slice(&quad.indices(len as u32));
               out.face.push(quad.quad.face);
               out.integer_pos.push(quad.quad.position);
          });
          out
     }

     pub fn quad_slice<'r>(&'r mut self, index: usize) -> RectilinearMeshSlice<'r>
     {
          let offset = index * 4;
          RectilinearMeshSlice {
               face: self.face[index],
               integer_position: self.integer_pos[index],
               pos: &mut self.pos[offset .. offset + 4],
               nor: &mut self.nor[offset .. offset + 4],
               uvs: &mut self.uvs[offset .. offset + 4],
               lum: &mut self.lum[offset .. offset + 4],
               aos: &mut self.aos[offset .. offset + 4],
          }
     }

     pub fn unit_cube() -> Self
     {
          Self::from_quads(&MeshQuad::cube())
     }

     pub fn scale(&mut self, scale: glam::Vec3)
     {
          self.pos.iter_mut().for_each(|pos| {
               *pos *= scale;
          });
     }

     pub fn shift(&mut self, shift: glam::Vec3)
     {
          self.pos.iter_mut().for_each(|pos| {
               *pos += shift;
          });
     }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct TerrainVertex
{
     pub pos: glam::Vec3,
     pub nor: glam::Vec3,
     pub tex: glam::Vec2,
     pub lum: f32,
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
          ];

          wgpu::VertexBufferLayout {
               array_stride: mem::size_of::<Self>() as u64,
               step_mode: wgpu::VertexStepMode::Vertex,
               attributes: ATTRIBS,
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkRawMesh
{
     pub vertices: Vec<TerrainVertex>,
     pub indices: Vec<u32>,
     pub offset: glam::IVec3,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkMesher<'c>
{
     pub view: &'c world::ChunkView,
     pub atlas: &'c atlas::TextureAtlas,
}

impl<'c> ChunkMesher<'c>
{
     pub fn to_rectilinear(&self) -> RectilinearMesh
     {
          use block::Visibility::*;

          let mut quads = Vec::new();

          let chunk = &self.view.chunk;
          let world_origin = chunk.world_position();
          let width = self.view.chunk_width;
          let height = self.view.chunk_height;

          for z in 0 .. width
          {
               for y in 0 .. height
               {
                    for x in 0 .. width
                    {
                         let coord = glam::ivec3(x, y, z);
                         let position = coord + world_origin;

                         let block = chunk.get(coord);
                         if block == &block::Block::EMPTY
                         {
                              continue;
                         }

                         for face in Face::CARDINALS
                         {
                              let offset = face.neighbor_offset();
                              let neighbor_coord = position + offset;
                              let neighbor = self.view.get_block(neighbor_coord);

                              let emit = match (block.visibility(), neighbor.visibility())
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

                              let ao = self.map_ao(position, face);
                              let lum = self.view.get_light(neighbor_coord) as f32 / light::MAX_LIGHT as f32;
                              match block.mesh_style()
                              {
                                   | block::EmittedMesh::RectilinearFull =>
                                   {
                                        quads.push(MeshQuad {
                                             quad: Quad { position, face },
                                             ao,
                                             lum: [lum; 4],
                                        });
                                   }
                                   | block::EmittedMesh::Decorator =>
                                   {
                                        quads.extend(Face::DIAGONALS.map(|face| {
                                             MeshQuad {
                                                  quad: Quad { position, face },
                                                  ao: [1.0; 4],
                                                  lum: [lum; 4],
                                             }
                                        }));
                                   }
                                   | block::EmittedMesh::RectilinearPartial => todo!(),
                              }
                         }
                    }
               }
          }

          RectilinearMesh::from_quads(&quads)
     }

     pub fn map_uvs(&self, rectilinear: &mut RectilinearMesh)
     {
          let chunk = &self.view.chunk;

          (0 .. rectilinear.size).for_each(|index| {
               let RectilinearMeshSlice { face, integer_position, uvs, .. } = rectilinear.quad_slice(index);

               let position = chunk.to_chunk_coords(integer_position);
               let block = chunk.get(position);
               self.atlas.conform_uvs(uvs, block.name(), face);
          });
     }

     pub fn map_ao(&self, coord: glam::IVec3, face: Face) -> [f32; 4]
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

               let side1 = self.view.get_block(adj + tn).opacity() != 0;
               let side2 = self.view.get_block(adj + btn).opacity() != 0;
               let corner = self.view.get_block(adj + tn + btn).opacity() != 0;

               let occlusion = match (side1, side2)
               {
                    | (true, true) => 0,
                    | _ => 3 - (side1 as i32 + side2 as i32 + corner as i32),
               };

               (occlusion as f32 + 1.0) * 0.25
          })
     }
}
