pub mod block;
pub mod chunk;
pub mod delta;
pub mod manager;
pub mod map;
pub mod physics;

use std::sync::{self};

use rustc_hash as rh;

use crate::visual::light;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkStage
{
     #[default]
     Allocated,
     TerrainGenerated,
     DecoratorsPlaced,
     LightingPropagated,
     LightingUpdated,
     Meshed,
}

#[derive(bon::Builder, Debug, Clone)]
pub struct ChunkView
{
     pub center: glam::IVec3,
     pub chunk: sync::Arc<chunk::Chunk>,
     pub neighbors: rh::FxHashMap<glam::IVec3, sync::Arc<chunk::Chunk>>,
     pub size: i32,
     pub chunk_width: i32,
     pub chunk_height: i32,
}

impl ChunkView
{
     pub fn resolve(&self, relative_coord: glam::IVec3) -> (glam::IVec3, glam::IVec3)
     {
          let rel = relative_coord.div_euclid(self.chunk.size());
          let local = relative_coord.rem_euclid(self.chunk.size());

          (rel, local)
     }

     pub fn get_block(&self, relative_coord: glam::IVec3) -> block::Block
     {
          let (rel, local) = self.resolve(relative_coord);
          match self.neighbors.get(&rel)
          {
               | Some(chunk) => *chunk.get(local),
               | None => block::Block::empty(),
          }
     }

     pub fn get_light(&self, relative_coord: glam::IVec3) -> light::Light
     {
          let (rel, local) = self.resolve(relative_coord);
          match self.neighbors.get(&rel)
          {
               | Some(chunk) => *chunk.get_light(local),
               | None => light::Light::min_light(),
          }
     }
}
