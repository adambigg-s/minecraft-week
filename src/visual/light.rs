use std::collections;
use std::ops;
use std::sync;

use crate::engine::neighbors;
use crate::world;
use crate::world::block;
use crate::world::chunk;
use crate::world::delta;

const MAX_LIGHT: LightInner = 15;

type LightInner = u8;

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
     pub inner: LightInner,
}

impl Light
{
     pub fn new(level: LightInner) -> Self
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
     T: Into<LightInner>,
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
     type Target = LightInner;

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

     pub fn remove_light(&mut self, light: LightNode)
     {
          self.remove_queue.push_back(light);
     }

     pub fn floodfill_lighting(
          &mut self,
          chunk: &mut chunk::Chunk,
          view: &world::ChunkView,
          deltas: &mut delta::LightDeltas,
     )
     {
          self.floodfill_negative(chunk, view, deltas);
          self.floodfill_positive(chunk, view, deltas);
     }

     fn floodfill_negative(
          &mut self,
          chunk: &mut chunk::Chunk,
          view: &world::ChunkView,
          deltas: &mut delta::LightDeltas,
     )
     {
          let mut boundaries = Vec::new();

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
                    let curr_light = view.get_light(coord);

                    if curr_light <= removal_light
                    {
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
                    }

                    continue;
               }

               let curr_block = *chunk.get(coord);
               let curr_light = *chunk.get_light(coord);
               let attenutation = *curr_block.opacity();
               let expected_light = Light::new(removal_light.saturating_sub(attenutation));

               if curr_light < expected_light
               {
                    continue;
               }
               if curr_light > expected_light
               {
                    boundaries.push(coord);
                    continue;
               }

               *chunk.get_light_mut(coord) = Light::min_light();

               neighbors::von_neumann3().into_iter().for_each(|(dx, dy, dz)| {
                    let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                    self.remove_queue.push_back(LightNode {
                         light: Light::new(expected_light.saturating_sub(1)),
                         coord: neighbor_coord,
                    });
               });
          }

          boundaries.into_iter().for_each(|coord| {
               let curr_light = *chunk.get_light(coord);
               if curr_light > Light::min_light()
               {
                    let outgoing_light = Light::new(curr_light.saturating_sub(1));
                    if outgoing_light > Light::min_light()
                    {
                         neighbors::von_neumann3().into_iter().for_each(|(dx, dy, dz)| {
                              let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                              self.add_queue.push_back(LightNode {
                                   light: outgoing_light,
                                   coord: neighbor_coord,
                              });
                         });
                    }
               }
          });
     }

     fn floodfill_positive(
          &mut self,
          chunk: &mut chunk::Chunk,
          view: &world::ChunkView,
          deltas: &mut delta::LightDeltas,
     )
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
                    let curr_light = view.get_light(coord);

                    if curr_light < light
                    {
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
                    }

                    continue;
               }

               let curr_block = *chunk.get(coord);
               let curr_light = *chunk.get_light(coord);
               let attenutation = *curr_block.opacity();
               let transmitted_light = Light::new(light.saturating_sub(attenutation));

               if curr_light >= light
               {
                    continue;
               }

               *chunk.get_light_mut(coord) = transmitted_light;

               neighbors::von_neumann3().into_iter().for_each(|(dx, dy, dz)| {
                    let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                    self.add_queue.push_back(LightNode {
                         light: Light::new(transmitted_light.saturating_sub(1)),
                         coord: neighbor_coord,
                    });
               });
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkLighting<'c>
{
     pub view: &'c world::ChunkView,
     pub ff: FloodFill,
}

impl<'c> ChunkLighting<'c>
{
     pub fn new(view: &'c world::ChunkView) -> Self
     {
          let ff = FloodFill::default();
          Self {
               view,
               ff,
          }
     }

     pub fn initialize_lighting(&mut self, chunk: &mut sync::Arc<chunk::Chunk>) -> delta::LightDeltas
     {
          let mut outgoing_deltas = delta::LightDeltas::new();
          let chunk = sync::Arc::make_mut(chunk);

          self.sky_lighting(chunk);

          self.ff.floodfill_lighting(chunk, self.view, &mut outgoing_deltas);

          outgoing_deltas
     }

     fn sky_lighting(&mut self, chunk: &mut chunk::Chunk)
     {
          for x in 0 .. self.view.chunk_width
          {
               for z in 0 .. self.view.chunk_width
               {
                    'height: for y in (0 .. self.view.chunk_height).rev()
                    {
                         let coord = glam::ivec3(x, y, z);
                         *chunk.get_light_mut(coord) = Light::max_light();
                         for (dx, dy, dz) in neighbors::von_neumann3()
                         {
                              let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                              if chunk.check_index(neighbor_coord)
                                   && *chunk.get_light(neighbor_coord) == Light::min_light()
                              {
                                   self.ff.add_light(
                                        LightNode {
                                             light: Light::new(3),
                                             coord: neighbor_coord,
                                        },
                                        chunk,
                                   );
                              }
                         }

                         let imm_down = coord + glam::ivec3(0, -1, 0);
                         if chunk.check_index(imm_down) && chunk.get(imm_down).opacity() != Light::min_light()
                         {
                              break 'height;
                         }
                    }
               }
          }
     }

     pub fn update_lighting(
          &mut self,
          chunk: &mut sync::Arc<chunk::Chunk>,
          deltas: &[delta::ChunkDelta<LightDelta>],
     ) -> delta::LightDeltas
     {
          let mut outgoing_deltas = delta::LightDeltas::new();
          let chunk = sync::Arc::make_mut(chunk);

          deltas.iter().for_each(|&delta| {
               let delta::ChunkDelta {
                    coord,
                    delta:
                         LightDelta {
                              light: sender_light,
                              removal,
                         },
               } = delta;

               let curr_light = *chunk.get_light(coord);

               if !removal && curr_light < sender_light
               {
                    self.ff.add_light(
                         LightNode {
                              light: sender_light,
                              coord,
                         },
                         chunk,
                    );
               }

               if removal && curr_light <= sender_light
               {
                    self.ff.remove_light(LightNode {
                         light: curr_light,
                         coord,
                    });
               }
               else if removal
               {
                    neighbors::von_neumann3().into_iter().for_each(|(dx, dy, dz)| {
                         let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                         let neighbor_light = self.view.get_light(neighbor_coord);
                         if neighbor_light > Light::min_light()
                         {
                              self.ff.add_light(
                                   LightNode {
                                        light: neighbor_light,
                                        coord: neighbor_coord,
                                   },
                                   chunk,
                              );
                         }
                    });
               }
          });

          self.ff.floodfill_lighting(chunk, self.view, &mut outgoing_deltas);

          outgoing_deltas
     }
}
