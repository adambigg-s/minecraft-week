use std::collections;
use std::sync;

use crate::engine::neighbors;
use crate::world;
use crate::world::block;

pub const MAX_LIGHT: u8 = 32;

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

     pub fn initial_lighting(&mut self)
     {
          let chunk = sync::Arc::make_mut(&mut self.view.chunk);

          for z in 0 .. self.view.chunk_width
          {
               for y in 0 .. self.view.chunk_height
               {
                    for x in 0 .. self.view.chunk_width
                    {
                         let coord = glam::ivec3(x, y, z);
                         if let Some(emissivitiy) = chunk.get(coord).emissivity()
                         {
                              *chunk.get_light_mut(coord) = emissivitiy;
                              self.queue.push_front(LightNode { light: emissivitiy, coord });
                         }
                    }
               }
          }

          for z in 0 .. self.view.chunk_width
          {
               for x in 0 .. self.view.chunk_width
               {
                    for y in (0 .. self.view.chunk_height).rev()
                    {
                         let coord = glam::ivec3(x, y, z);
                         let block = chunk.get(coord);

                         if block != &block::Block::EMPTY
                         {
                              if block.emissivity().is_none()
                              {
                                   *chunk.get_light_mut(coord) = MAX_LIGHT;
                                   self.queue.push_front(LightNode { light: MAX_LIGHT, coord });
                              }
                              break;
                         }

                         *chunk.get_light_mut(coord) = MAX_LIGHT;
                    }
               }
          }

          log::debug!("Number of nodes in light queue: {}", self.queue.len());
          self.floodfill();
     }

     pub fn add_light(&mut self, node: LightNode)
     {
          todo!()
     }

     pub fn remove_light(&mut self, node: LightNode)
     {
          todo!()
     }

     fn floodfill(&mut self)
     {
          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          while let Some(node) = self.queue.pop_back()
          {
               let light = node.light.saturating_sub(1);
               if light == 0
               {
                    continue;
               }

               for (dx, dy, dz) in neighbors::von_neumann3()
               {
                    let coord = node.coord + glam::ivec3(dx, dy, dz);
                    if !chunk.check_index(coord)
                    {
                         continue;
                    }

                    let neighbor = chunk.get(coord);

                    let opacity = neighbor.opacity();
                    let transmitted = node.light.saturating_sub(opacity + 1);

                    if transmitted > *chunk.get_light(coord)
                    {
                         *chunk.get_light_mut(coord) = transmitted;
                         self.queue.push_front(LightNode { light: transmitted, coord });
                    }
               }
          }
     }
}
