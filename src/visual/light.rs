#![allow(clippy::let_and_return)]
#![allow(unused)]

use std::collections;
use std::ops;

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

#[derive(bon::Builder, Debug)]
pub struct ChunkLighting<'c>
{
     pub view: &'c mut world::ChunkView,
     pub add_queue: collections::VecDeque<LightNode>,
     pub remove_queue: collections::VecDeque<LightNode>,
}

impl<'c> ChunkLighting<'c>
{
     pub fn new(view: &'c mut world::ChunkView) -> Self
     {
          Self {
               view,
               add_queue: collections::VecDeque::new(),
               remove_queue: collections::VecDeque::new(),
          }
     }

     pub fn initialize_lighting(&mut self) -> delta::LightDeltas
     {
          let deltas = delta::LightDeltas::new();

          deltas
     }

     pub fn update_lighting(&mut self, deltas: &[delta::ChunkDelta<Light>]) -> delta::LightDeltas
     {
          let deltas_out = delta::LightDeltas::new();

          deltas_out
     }
}
