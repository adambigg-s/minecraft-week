use std::collections;
use std::ops;
use std::sync;

use crate::engine::neighbors;
use crate::world;
use crate::world::block;
use crate::world::delta;

const MAX_LIGHT: u8 = 15;

#[repr(transparent)]
#[derive(bon::Builder, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Light
{
     pub inner: u8,
}

impl Light
{
     pub fn new(level: u8) -> Self
     {
          Self {
               inner: level,
          }
     }

     pub fn min_light() -> Self
     {
          Self {
               inner: 0,
          }
     }

     pub fn max_light() -> Self
     {
          MAX_LIGHT.into()
     }
}

impl<T> From<T> for Light
where
     T: Into<u8>,
{
     fn from(value: T) -> Self
     {
          Self {
               inner: value.into(),
          }
     }
}

impl ops::Deref for Light
{
     type Target = u8;

     fn deref(&self) -> &Self::Target
     {
          &self.inner
     }
}

impl ops::DerefMut for Light
{
     fn deref_mut(&mut self) -> &mut Self::Target
     {
          &mut self.inner
     }
}

#[derive(bon::Builder, Debug)]
pub struct LightNode
{
     pub light: Light,
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
          Self {
               view,
               queue: collections::VecDeque::new(),
          }
     }

     pub fn initial_lighting(&mut self) -> delta::LightDeltas
     {
          let mut deltas = delta::LightDeltas::new();

          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          chunk.lights_mut().fill(Light::min_light());

          for z in 0 .. self.view.chunk_width
          {
               for x in 0 .. self.view.chunk_width
               {
                    'height: for y in (0 .. self.view.chunk_height).rev()
                    {
                         let coord = glam::ivec3(x, y, z);
                         *chunk.get_light_mut(coord) = Light::max_light();

                         for (dx, dy, dz) in neighbors::von_neumann3()
                         {
                              let coord = coord + glam::ivec3(dx, dy, dz);
                              self.queue.push_front(LightNode {
                                   light: Light::max_light(),
                                   coord,
                              });
                         }

                         let imm_down = coord + glam::ivec3(0, -1, 0);
                         if chunk.check_index(imm_down)
                              && chunk.get(imm_down).visibility() == block::Visibility::Opaque
                         {
                              break 'height;
                         }
                    }
               }
          }

          for z in 0 .. self.view.chunk_width
          {
               for y in 0 .. self.view.chunk_height
               {
                    for x in 0 .. self.view.chunk_width
                    {
                         let coord = glam::ivec3(x, y, z);
                         if let Some(emissivitiy) = chunk.get(coord).emissivity()
                         {
                              self.queue.push_front(LightNode {
                                   light: emissivitiy,
                                   coord,
                              });
                         }
                    }
               }
          }

          self.floodfill(&mut deltas);

          deltas
     }

     pub fn update_lighting(&mut self, deltas: Vec<delta::ChunkDelta<Light>>) -> delta::LightDeltas
     {
          deltas.into_iter().for_each(|delta| {
               self.queue.push_front(LightNode {
                    light: delta.delta,
                    coord: delta.coord,
               });
          });

          let mut deltas = delta::LightDeltas::new();
          self.floodfill(&mut deltas);
          deltas
     }

     fn floodfill(&mut self, deltas: &mut delta::LightDeltas)
     {
          while let Some(LightNode {
               mut light,
               coord,
          }) = self.queue.pop_back()
          {
               let chunk = &self.view.chunk;

               if *light == 0
               {
                    continue;
               }

               if !chunk.check_index(coord)
               {
                    self.queue_delta(deltas, chunk, light, coord);
                    continue;
               }

               let curr_block = *chunk.get(coord);
               let curr_light = *chunk.get_light(coord);

               let attenuation = curr_block.opacity();
               *light = light.saturating_sub(1 + *attenuation);

               let chunk = sync::Arc::make_mut(&mut self.view.chunk);
               if curr_light < light
               {
                    *chunk.get_light_mut(coord) = light;

                    for (dx, dy, dz) in neighbors::von_neumann3()
                    {
                         let coord = coord + glam::ivec3(dx, dy, dz);
                         self.queue.push_front(LightNode {
                              light,
                              coord,
                         });
                    }
               }
          }
     }

     fn queue_delta(
          &self,
          deltas: &mut delta::ChunkDeltaMap<Light>,
          chunk: &sync::Arc<world::chunk::Chunk>,
          light: Light,
          coord: glam::IVec3,
     )
     {
          let world_coord = coord + chunk.world_position();
          let world = chunk.chunk_world_coords(world_coord);
          let chunk = chunk.to_chunk_coords(world_coord);

          if self.view.get_light(world_coord) < light
          {
               deltas.insert(
                    world,
                    delta::ChunkDelta {
                         coord: chunk,
                         delta: light,
                    },
               );
          }
     }
}
