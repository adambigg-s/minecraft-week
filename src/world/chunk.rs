use std::ops;

use crate::engine::kinematics;
use crate::engine::storage::buffer;
use crate::visual::atlas;
use crate::visual::mesher;
use crate::world;
use crate::world::block;

#[derive(bon::Builder, Debug, Clone)]
pub struct Chunk
{
     blocks: buffer::Buffer<block::Block, 3>,
     lights: buffer::Buffer<u8, 3>,
     offset: glam::IVec3,
     height: usize,
     width: usize,
}

impl Chunk
{
     pub fn new(offset: glam::IVec3, width: usize, height: usize) -> Self
     {
          let blocks = buffer::Buffer::new_zeroed([width, height, width]);
          let lights = buffer::Buffer::new_zeroed([width, height, width]);

          Self { blocks, lights, offset, height, width }
     }

     pub fn width(&self) -> usize
     {
          self.width
     }

     pub fn height(&self) -> usize
     {
          self.height
     }

     pub fn offset(&self) -> glam::IVec3
     {
          self.offset
     }

     pub fn indices(&self) -> ops::Range<usize>
     {
          0..self.blocks.size().iter().product()
     }

     pub fn delinearize(&self, index: usize) -> [usize; 3]
     {
          self.blocks.delinearize(index)
     }

     pub fn size(&self) -> glam::IVec3
     {
          glam::ivec3(self.width as i32, self.height as i32, self.width as i32)
     }

     pub fn world_position(&self) -> glam::IVec3
     {
          self.size() * self.offset
     }

     pub fn to_index(&self, coord: glam::IVec3) -> [usize; 3]
     {
          coord.to_array().map(|ele| ele as usize)
     }

     pub fn check_index(&self, coord: glam::IVec3) -> bool
     {
          let index = self.to_index(coord);
          self.blocks.surrounds(index)
     }

     pub fn to_chunk_coords(&self, world_coord: glam::IVec3) -> glam::IVec3
     {
          world_coord.rem_euclid(self.size())
     }

     pub fn chunk_world_coords(&self, world_coord: glam::IVec3) -> glam::IVec3
     {
          world_coord.div_euclid(self.size())
     }

     pub fn get(&self, coord: glam::IVec3) -> &block::Block
     {
          self.blocks.get(self.to_index(coord))
     }

     pub fn get_mut(&mut self, coord: glam::IVec3) -> &mut block::Block
     {
          self.blocks.get_mut(self.to_index(coord))
     }

     pub fn get_light(&self, coord: glam::IVec3) -> &u8
     {
          self.lights.get(self.to_index(coord))
     }

     pub fn get_light_mut(&mut self, coord: glam::IVec3) -> &mut u8
     {
          self.lights.get_mut(self.to_index(coord))
     }

     pub fn raw_mesh(&self, atlas: &atlas::TextureAtlas, view: &world::ChunkView) -> mesher::ChunkRawMesh
     {
          let mesher = mesher::ChunkMesher { view, atlas };
          let mut rectilinear = mesher.to_rectilinear();
          mesher.map_uvs(&mut rectilinear);

          let mut vertices = Vec::new();
          (0..rectilinear.size).for_each(|index| {
               let mesher::RectilinearMeshSlice { pos, nor, uvs, lum, aos, .. } =
                    rectilinear.quad_slice(index);

               (0..4).for_each(|vertex| {
                    vertices.push(mesher::TerrainVertex {
                         pos: pos[vertex],
                         nor: nor[vertex],
                         tex: uvs[vertex],
                         lum: lum[vertex],
                         ao: aos[vertex],
                    });
               });
          });
          let indices = rectilinear.index;
          let offset = self.offset;

          mesher::ChunkRawMesh { vertices, indices, offset }
     }
}

impl kinematics::Collision for Chunk
{
     type Collider = kinematics::BoxCollider;

     fn collides(&self, collider: Self::Collider) -> bool
     {
          let mins = collider.lo.map(|val| val.floor() as i32);
          let maxs = collider.hi.map(|val| val.ceil() as i32);
          for z in mins[2]..maxs[2]
          {
               for y in mins[1]..maxs[1]
               {
                    for x in mins[0]..maxs[0]
                    {
                         let coord = glam::ivec3(x, y, z);
                         let target_chunk = glam::ivec3(
                              (coord.x as f32 / self.width as f32).floor() as i32,
                              0,
                              (coord.z as f32 / self.width as f32).floor() as i32,
                         );
                         if target_chunk != self.offset
                         {
                              continue;
                         }

                         let chunk_coord = self.to_chunk_coords(coord);
                         if !self.check_index(chunk_coord)
                         {
                              continue;
                         }
                         if self.get(chunk_coord).collides(())
                         {
                              return true;
                         }
                    }
               }
          }

          false
     }
}
