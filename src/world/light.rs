use std::collections;
use std::sync;

use crate::world;

pub const MAX_LIGHT: u8 = 25;

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
          for z in 0 .. self.view.chunk.width() as i32
          {
               for y in 0 .. self.view.chunk.height() as i32
               {
                    for x in 0 .. self.view.chunk.width() as i32
                    {
                         let coord = glam::ivec3(x, y, z);
                         if self.view.chunk.get_light(coord) == &0
                         {
                              continue;
                         }
                         self.floodfill(coord);
                    }
               }
          }
     }

     fn floodfill(&mut self, coord: glam::IVec3)
     {
          let light = *self.view.chunk.get_light(coord);
          self.queue.push_front(LightNode { light, coord });

          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          while let Some(node) = self.queue.pop_back()
          {
               if node.light <= 1
               {
                    continue;
               }

               let new_light = node.light - 1;
               for (dx, dy, dz) in [
                    (1, 0, 0),
                    (-1, 0, 0),
                    (0, 1, 0),
                    (0, -1, 0),
                    (0, 0, 1),
                    (0, 0, -1),
               ]
               {
                    let neighbor = node.coord + glam::ivec3(dx, dy, dz);

                    if chunk.check_index(neighbor)
                    {
                         let curr_light = *chunk.get_light(neighbor);

                         if new_light - 1 > curr_light
                         {
                              *chunk.get_light_mut(neighbor) = new_light;
                              self.queue.push_front(LightNode { light: new_light, coord: neighbor });
                         }
                    }
               }
          }
     }
}
