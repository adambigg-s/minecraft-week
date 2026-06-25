use std::collections;
use std::ops;
use std::sync;

use crate::engine::neighbors;
use crate::world;
use crate::world::block;
use crate::world::chunk;
use crate::world::delta;

const MAX_LIGHT: u8 = 8;

#[derive(bon::Builder, Debug, Default, Clone, Copy)]
pub struct LightDelta
{
     pub light: Light,
     pub removal: bool,
}

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

     pub fn luminosity(self) -> f32
     {
          *self as f32 / *Self::max_light() as f32
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

#[derive(bon::Builder, Debug, Default)]
pub struct FloodFill
{
     pub add_queue: collections::VecDeque<LightNode>,
     pub remove_queue: collections::VecDeque<LightNode>,
     pub visited: collections::HashSet<glam::IVec3>,
}

impl FloodFill
{
     pub fn add_light(&mut self, light: LightNode, chunk: &mut chunk::Chunk)
     {
          if chunk.check_index(light.coord)
          {
               *chunk.get_light_mut(light.coord) = Light::min_light();
          }
          self.add_queue.push_back(light);
     }

     pub fn remove_light(&mut self, light: LightNode, _: &mut chunk::Chunk)
     {
          self.remove_queue.push_back(light);
     }

     pub fn floodlight(&mut self, chunk: &mut chunk::Chunk, deltas: &mut delta::LightDeltas)
     {
          self.floodfill_negative(chunk, deltas);

          self.floodfill_positive(chunk, deltas);
     }

     fn floodfill_negative(&mut self, chunk: &mut chunk::Chunk, deltas: &mut delta::LightDeltas)
     {
          self.visited.clear();
          while let Some(node) = self.remove_queue.pop_front()
          {
               let LightNode {
                    light: removal_light,
                    coord,
               } = node;

               if removal_light == Light::min_light()
               {
                    continue;
               }

               if !chunk.check_index(coord)
               {
                    let global_pos = chunk.world_position() + coord;
                    let world_coord = chunk.chunk_world_coords(global_pos);
                    let chunk_coord = chunk.to_chunk_coords(global_pos);
                    deltas.insert(
                         world_coord,
                         delta::ChunkDelta {
                              coord: chunk_coord,
                              delta: LightDelta {
                                   light: removal_light,
                                   removal: true,
                              },
                         },
                    );

                    continue;
               }

               let curr_block = *chunk.get(coord);
               let curr_light = *chunk.get_light(coord);

               let attenutation = *curr_block.opacity();
               let transmitted_light = Light::new(removal_light.saturating_sub(attenutation));
               *chunk.get_light_mut(coord) = Light::min_light();

               if transmitted_light > removal_light
               {
                    self.add_light(
                         LightNode {
                              light: transmitted_light,
                              coord,
                         },
                         chunk,
                    );

                    continue;
               }

               for (dx, dy, dz) in neighbors::von_neumann3()
               {
                    let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                    if self.visited.insert(neighbor_coord)
                    {
                         self.remove_queue.push_back(LightNode {
                              light: Light::new(transmitted_light.saturating_sub(1)),
                              coord: neighbor_coord,
                         });
                    }
               }
          }
     }

     fn floodfill_positive(&mut self, chunk: &mut chunk::Chunk, deltas: &mut delta::LightDeltas)
     {
          while let Some(node) = self.add_queue.pop_front()
          {
               let LightNode {
                    light,
                    coord,
               } = node;

               if light == Light::min_light()
               {
                    continue;
               }

               if !chunk.check_index(coord)
               {
                    let global_pos = chunk.world_position() + coord;
                    let world_coord = chunk.chunk_world_coords(global_pos);
                    let chunk_coord = chunk.to_chunk_coords(global_pos);
                    deltas.insert(
                         world_coord,
                         delta::ChunkDelta {
                              coord: chunk_coord,
                              delta: LightDelta {
                                   light,
                                   removal: false,
                              },
                         },
                    );

                    continue;
               }

               let curr_block = *chunk.get(coord);
               let curr_light = *chunk.get_light(coord);
               if curr_light >= light
               {
                    continue;
               }

               let attenutation = *curr_block.opacity();
               let transmitted_light = Light::new(light.saturating_sub(attenutation));
               *chunk.get_light_mut(coord) = transmitted_light;
               for (dx, dy, dz) in neighbors::von_neumann3()
               {
                    let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                    self.add_queue.push_back(LightNode {
                         light: Light::new(transmitted_light.saturating_sub(1)),
                         coord: neighbor_coord,
                    });
               }
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkLighting<'c>
{
     pub view: &'c mut world::ChunkView,
     pub ff: FloodFill,
}

impl<'c> ChunkLighting<'c>
{
     pub fn new(view: &'c mut world::ChunkView) -> Self
     {
          let ff = FloodFill::default();
          Self {
               view,
               ff,
          }
     }

     pub fn initialize_lighting(&mut self) -> delta::LightDeltas
     {
          let mut outgoing_deltas = delta::LightDeltas::new();

          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          self.ff.floodlight(chunk, &mut outgoing_deltas);

          outgoing_deltas
     }

     fn sky_lighting(&mut self)
     {
          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          for z in 0 .. self.view.chunk_width
          {
               for x in 0 .. self.view.chunk_width
               {
                    'height: for y in (0 .. self.view.chunk_height).rev()
                    {
                         let coord = glam::ivec3(x, y, z);
                         self.ff.add_light(
                              LightNode {
                                   light: Light::max_light(),
                                   coord,
                              },
                              chunk,
                         );

                         let imm_down = coord + glam::ivec3(0, -1, 0);
                         if chunk.check_index(imm_down)
                              && chunk.get(imm_down).visibility() != block::Visibility::Invisible
                         {
                              break 'height;
                         }
                    }
               }
          }
     }

     pub fn update_lighting(&mut self, deltas: &[delta::ChunkDelta<LightDelta>]) -> delta::LightDeltas
     {
          let mut outgoing_deltas = delta::LightDeltas::new();

          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          for &delta in deltas
          {
               let delta::ChunkDelta {
                    coord,
                    delta:
                         LightDelta {
                              light,
                              removal,
                         },
               } = delta;
               debug_assert!(chunk.check_index(coord));

               let curr_light = *chunk.get_light(coord);
               let curr_block = *chunk.get(coord);
               let intrinsic_light = curr_block.emissivity().unwrap_or(Light::min_light());

               let expected_light = light.max(intrinsic_light);

               if curr_light >= expected_light && removal
               {
                    log::info!(
                         "Incoming light removal: coord={:?}, curr={:?}, incoming={:?}",
                         coord,
                         curr_light,
                         light
                    );
                    self.ff.remove_light(
                         LightNode {
                              light: curr_light,
                              coord,
                         },
                         chunk,
                    );
               }
               else if curr_light < expected_light && !removal
               {
                    log::info!("Light Addition: {:?}, {:?}", coord, light);
                    self.ff.add_light(
                         LightNode {
                              light: expected_light,
                              coord,
                         },
                         chunk,
                    );
               }
          }

          self.ff.floodlight(chunk, &mut outgoing_deltas);

          outgoing_deltas
     }
}
