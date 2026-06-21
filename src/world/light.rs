use std::collections;
use std::sync;

use crate::engine::neighbors;
use crate::world;
use crate::world::block;

pub const MAX_LIGHT: u8 = 15;

#[derive(bon::Builder, Debug)]
pub struct LightNode
{
     pub light: u8,
     pub coord: glam::IVec3,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkLighting<'c>
{
     pub view: &'c mut world::ChunkView,
     pub queue: collections::VecDeque<LightNode>,
}

impl<'c> ChunkLighting<'c>
{
     pub fn new(view: &'c mut world::ChunkView) -> Self
     {
          Self { view, queue: collections::VecDeque::new() }
     }

     pub fn lighting(&mut self)
     {
          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          for z in 0 .. self.view.chunk_width
          {
               for x in 0 .. self.view.chunk_width
               {
                    for y in (0 .. self.view.chunk_height).rev()
                    {
                         let coord = glam::ivec3(x, y, z);
                         *chunk.get_light_mut(coord) = MAX_LIGHT;

                         if chunk.get(coord).visibility() != block::Visibility::Invisible
                         {
                              self.queue.push_front(LightNode { light: MAX_LIGHT, coord });
                              break;
                         }
                    }
               }
          }
          self.floodfill();
     }

     fn floodfill(&mut self)
     {
          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          while let Some(node) = self.queue.pop_back()
          {
               *chunk.get_light_mut(node.coord) = node.light;

               if node.light <= 1
               {
                    continue;
               }

               let light = node.light - 1;
               for (dx, dy, dz) in neighbors::von_neumann3()
               {
                    let coord = node.coord + glam::ivec3(dx, dy, dz);
                    if !chunk.check_index(coord)
                    {
                         continue;
                    }

                    if chunk.get(coord).visibility() != block::Visibility::Invisible
                    {
                         continue;
                    }

                    if light - 1 > *chunk.get_light(coord)
                    {
                         self.queue.push_front(LightNode { light, coord });
                    }
               }
          }
     }
}
