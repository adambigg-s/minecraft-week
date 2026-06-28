use std::ops;

use crate::engine::storage::buffer;
use crate::visual::atlas;
use crate::visual::light;
use crate::visual::mesher;
use crate::world;
use crate::world::block;

#[derive(bon::Builder, Debug, Clone)]
pub struct Chunk
{
     blocks: buffer::Buffer<block::Block, 3>,
     lights: buffer::Buffer<light::Light, 3>,
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

          Self {
               blocks,
               lights,
               offset,
               height,
               width,
          }
     }

     pub fn blocks_mut(&mut self) -> &mut buffer::Buffer<block::Block, 3>
     {
          &mut self.blocks
     }

     pub fn lights_mut(&mut self) -> &mut buffer::Buffer<light::Light, 3>
     {
          &mut self.lights
     }

     pub fn offset(&self) -> glam::IVec3
     {
          self.offset
     }

     pub fn width(&self) -> usize
     {
          self.width
     }

     pub fn height(&self) -> usize
     {
          self.height
     }

     pub fn indices(&self) -> ops::Range<usize>
     {
          0 .. self.blocks.size().iter().product()
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

     pub fn get_light(&self, coord: glam::IVec3) -> &light::Light
     {
          self.lights.get(self.to_index(coord))
     }

     pub fn get_light_mut(&mut self, coord: glam::IVec3) -> &mut light::Light
     {
          self.lights.get_mut(self.to_index(coord))
     }

     pub fn raw_mesh(&self, atlas: &atlas::TextureAtlas, view: &world::ChunkView) -> mesher::ChunkRawMesh
     {
          mesher::ChunkMesher {
               view,
               atlas,
          }
          .raw_opaque_mesh()
     }
}
