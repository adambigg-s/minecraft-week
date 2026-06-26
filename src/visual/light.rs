use std::collections;
use std::ops;
use std::sync;

use crate::engine::neighbors;
use crate::world;
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

     pub fn remove_light(&mut self, light: LightNode, chunk: &mut chunk::Chunk)
     {
          if chunk.check_index(light.coord)
          {
               // *chunk.get_light_mut(light.coord) = Light::min_light();
          }
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
                    self.add_light(
                         LightNode {
                              light: curr_light,
                              coord,
                         },
                         chunk,
                    );
               }

               *chunk.get_light_mut(coord) = Light::min_light();

               for (dx, dy, dz) in neighbors::von_neumann3()
               {
                    let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                    self.remove_queue.push_back(LightNode {
                         light: Light::new(expected_light.saturating_sub(1)),
                         coord: neighbor_coord,
                    });
               }
          }
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

     #[allow(clippy::let_and_return)]
     pub fn initialize_lighting(&mut self, chunk: &mut sync::Arc<chunk::Chunk>) -> delta::LightDeltas
     {
          let outgoing_deltas = delta::LightDeltas::new();

          outgoing_deltas
     }

     pub fn update_lighting(
          &mut self,
          chunk: &mut sync::Arc<chunk::Chunk>,
          deltas: &[delta::ChunkDelta<LightDelta>],
     ) -> delta::LightDeltas
     {
          let mut outgoing_deltas = delta::LightDeltas::new();
          let chunk = sync::Arc::make_mut(chunk);

          for &delta in deltas
          {
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
                    log::info!("Light addition triggered: {:?}", delta);
                    self.ff.add_light(
                         LightNode {
                              light: sender_light,
                              coord,
                         },
                         chunk,
                    );
               }

               if removal
               {
                    log::info!("Light removal triggered: {:?}", delta);
                    if curr_light <= sender_light
                    {
                         self.ff.remove_light(
                              LightNode {
                                   light: curr_light,
                                   coord,
                              },
                              chunk,
                         );
                    }
               }
          }

          self.ff.floodfill_lighting(chunk, self.view, &mut outgoing_deltas);

          outgoing_deltas
     }
}

// fn floodfill_negative(
//      &mut self,
//      chunk: &mut chunk::Chunk,
//      view: &world::ChunkView,
//      deltas: &mut delta::LightDeltas,
// )
// {
//      self.visited.clear();
//      while let Some(node) = self.remove_queue.pop_front()
//      {
//           let LightNode {
//                light: removal_light,
//                coord,
//           } = node;

//           if removal_light == Light::min_light()
//           {
//                continue;
//           }

//           if !chunk.check_index(coord)
//           {
//                let global_pos = chunk.world_position() + coord;
//                let world_coord = chunk.chunk_world_coords(global_pos);
//                let chunk_coord = chunk.to_chunk_coords(global_pos);
//                if view.get_light(coord) < removal_light
//                {
//                     deltas.insert(
//                          world_coord,
//                          delta::ChunkDelta {
//                               coord: chunk_coord,
//                               delta: LightDelta {
//                                    light: removal_light,
//                                    removal: true,
//                               },
//                          },
//                     );
//                }

//                continue;
//           }

//           let curr_block = *chunk.get(coord);
//           let curr_light = *chunk.get_light(coord);

//           let attenutation = *curr_block.opacity();
//           let transmitted_light = Light::new(removal_light.saturating_sub(attenutation));
//           *chunk.get_light_mut(coord) = Light::min_light();

//           if transmitted_light > removal_light
//           {
//                self.add_light(
//                     LightNode {
//                          light: transmitted_light,
//                          coord,
//                     },
//                     chunk,
//                );

//                continue;
//           }

//           for (dx, dy, dz) in neighbors::von_neumann3()
//           {
//                let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
//                if self.visited.insert(neighbor_coord)
//                {
//                     self.remove_queue.push_back(LightNode {
//                          light: Light::new(transmitted_light.saturating_sub(1)),
//                          coord: neighbor_coord,
//                     });
//                }
//           }
//      }
// }

// THIS ONE IS WORKING, IT JUST CRASHES ON OCCCASION
// fn floodfill_negative(
//      &mut self,
//      chunk: &mut chunk::Chunk,
//      view: &world::ChunkView,
//      deltas: &mut delta::LightDeltas,
// )
// {
//      self.visited.clear();

//      while let Some(node) = self.remove_queue.pop_front()
//      {
//           let LightNode {
//                light: removal_light,
//                coord,
//           } = node;

//           if removal_light == Light::min_light()
//           {
//                continue;
//           }

//           for (dx, dy, dz) in neighbors::von_neumann3()
//           {
//                let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
//                if !self.visited.insert(neighbor_coord)
//                {
//                     continue;
//                }

//                let inbounds = chunk.check_index(neighbor_coord);

//                if inbounds
//                {
//                     let neighbor_light = *chunk.get_light(neighbor_coord);
//                     if neighbor_light < removal_light
//                     {
//                          *chunk.get_light_mut(neighbor_coord) = Light::min_light();
//                          self.remove_queue.push_back(LightNode {
//                               light: neighbor_light,
//                               coord: neighbor_coord,
//                          });
//                     }
//                     else if neighbor_light >= removal_light
//                     {
//                          *chunk.get_light_mut(neighbor_coord) = Light::min_light();
//                          self.add_queue.push_back(LightNode {
//                               light: neighbor_light,
//                               coord: neighbor_coord,
//                          });
//                     }
//                }
//                else
//                {
//                     let global_pos = chunk.world_position() + neighbor_coord;
//                     let world_coord = chunk.chunk_world_coords(global_pos);
//                     let chunk_coord = chunk.to_chunk_coords(global_pos);
//                     let curr_light = view.get_light(chunk_coord);

//                     if curr_light > removal_light
//                     {
//                          continue;
//                     }

//                     deltas.insert(
//                          world_coord,
//                          delta::ChunkDelta {
//                               coord: chunk_coord,
//                               delta: LightDelta {
//                                    light: removal_light,
//                                    removal: true,
//                               },
//                          },
//                     );
//                }
//           }
//      }
// }
