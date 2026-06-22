use std::sync;

use rustc_hash as rh;

use crate::world::chunk;
use crate::world::{self};

#[derive(bon::Builder, Debug)]
pub struct ChunkEntry
{
     pub stage: world::ChunkStage,
     pub chunk: sync::Arc<chunk::Chunk>,
}

#[derive(bon::Builder, Debug, Default)]
pub struct ChunkMap
{
     pub chunks: sync::RwLock<rh::FxHashMap<glam::IVec3, ChunkEntry>>,
     pub update_times: rh::FxHashMap<glam::IVec3, f32>,
}

impl ChunkMap
{
     pub fn new() -> Self
     {
          Self::default()
     }

     pub fn insert(&self, coord: glam::IVec3, chunk: sync::Arc<chunk::Chunk>)
     {
          self.chunks.write().unwrap().insert(
               coord,
               ChunkEntry {
                    stage: world::ChunkStage::Allocated,
                    chunk,
               },
          );
     }

     pub fn remove(&self, coord: &glam::IVec3)
     {
          self.chunks.write().unwrap().remove(coord);
     }

     pub fn contains(&self, coord: &glam::IVec3) -> bool
     {
          self.chunks.read().unwrap().contains_key(coord)
     }

     pub fn set_stage(&self, coord: &glam::IVec3, stage: world::ChunkStage) -> bool
     {
          if let Some(chunk) = self.chunks.write().unwrap().get_mut(coord)
          {
               chunk.stage = stage;
               return true;
          }
          false
     }

     pub fn get_stage(&self, coord: &glam::IVec3) -> Option<world::ChunkStage>
     {
          if let Some(chunk) = self.chunks.read().unwrap().get(coord)
          {
               return Some(chunk.stage);
          }
          None
     }

     pub fn set_time(&mut self, coord: &glam::IVec3, time: f32)
     {
          *self.update_times.entry(*coord).or_default() = time
     }

     pub fn get_time(&self, coord: &glam::IVec3) -> Option<f32>
     {
          if let Some(&curr_time) = self.update_times.get(coord)
          {
               return Some(curr_time);
          }
          None
     }

     pub fn get_complete_view(
          &self,
          center: glam::IVec3,
          stage_threshold: world::ChunkStage,
          size: i32,
     ) -> Option<world::ChunkView>
     {
          let map = self.chunks.read().unwrap();
          let mut neighbors = rh::FxHashMap::default();
          for dz in -size ..= size
          {
               for dx in -size ..= size
               {
                    let rel = glam::ivec3(dx, 0, dz);
                    let coord = center + rel;

                    let chunk = map.get(&coord)?;
                    if chunk.stage < stage_threshold
                    {
                         return None;
                    }

                    neighbors.insert(rel, sync::Arc::clone(&chunk.chunk));
               }
          }
          let chunk = sync::Arc::clone(&map[&center].chunk);
          let chunk_width = chunk.width() as i32;
          let chunk_height = chunk.height() as i32;

          Some(world::ChunkView {
               center,
               chunk,
               neighbors,
               size,
               chunk_width,
               chunk_height,
          })
     }

     pub fn get_any_view(
          &self,
          center: glam::IVec3,
          stage_threshold: world::ChunkStage,
          size: i32,
     ) -> Option<world::ChunkView>
     {
          let map = self.chunks.read().unwrap();
          let mut neighbors = rh::FxHashMap::default();
          for dz in -size ..= size
          {
               for dx in -size ..= size
               {
                    let rel = glam::ivec3(dx, 0, dz);
                    let coord = center + rel;

                    let chunk = map.get(&coord);
                    if let Some(chunk) = chunk
                    {
                         if chunk.stage < stage_threshold
                         {
                              continue;
                         }

                         neighbors.insert(rel, sync::Arc::clone(&chunk.chunk));
                    }
               }
          }
          let chunk = sync::Arc::clone(&map.get(&center)?.chunk);
          let chunk_width = chunk.width() as i32;
          let chunk_height = chunk.height() as i32;

          Some(world::ChunkView {
               center,
               chunk,
               neighbors,
               size,
               chunk_width,
               chunk_height,
          })
     }
}
