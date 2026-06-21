use std::collections;
use std::ops;
use std::sync;

use crate::engine::neighbors;
use crate::world;
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
               for y in 0 .. self.view.chunk_height
               {
                    for x in 0 .. self.view.chunk_width
                    {
                         let coord = glam::ivec3(x, y, z);
                         if let Some(emissivitiy) = chunk.get(coord).emissivity()
                         {
                              *chunk.get_light_mut(coord) = emissivitiy;
                              self.queue.push_front(LightNode {
                                   light: emissivitiy,
                                   coord,
                              });
                         }
                    }
               }
          }

          log::debug!("Number of nodes in light queue: {}", self.queue.len());
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

     #[allow(unused)]
     pub fn add_light(&mut self, node: LightNode)
     {
          todo!()
     }

     #[allow(unused)]
     pub fn remove_light(&mut self, node: LightNode)
     {
          todo!()
     }

     fn floodfill(&mut self, deltas: &mut delta::LightDeltas)
     {
          let chunk = sync::Arc::make_mut(&mut self.view.chunk);
          while let Some(node) = self.queue.pop_back()
          {
               let light = Light::new(node.light.saturating_sub(1));
               if *light == 0
               {
                    continue;
               }

               for (dx, dy, dz) in neighbors::von_neumann3()
               {
                    let coord = node.coord + glam::ivec3(dx, dy, dz);
                    if !chunk.check_index(coord)
                    {
                         let global_coords = coord + chunk.world_position();
                         let world = chunk.chunk_world_coords(global_coords);
                         let chunk = chunk.to_chunk_coords(global_coords);
                         deltas.insert(
                              world,
                              delta::ChunkDelta {
                                   coord: chunk,
                                   delta: light,
                              },
                         );

                         continue;
                    }

                    let opacity = chunk.get(coord).opacity();
                    let transmitted = Light::new(node.light.saturating_sub(*opacity + 1));
                    if transmitted > *chunk.get_light(coord)
                    {
                         *chunk.get_light_mut(coord) = transmitted;
                         self.queue.push_front(LightNode {
                              light: transmitted,
                              coord,
                         });
                    }
               }
          }
     }
}
