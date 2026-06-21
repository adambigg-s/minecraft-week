use rustc_hash as rh;

use crate::world::block;
use crate::world::chunk;
use crate::world::light;

pub trait DeltaValue
{
     fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
     where
          Self: Sized;
}

#[derive(bon::Builder, Debug, Default, Clone, Copy)]
pub struct ChunkDelta<T>
{
     pub coord: glam::IVec3,
     pub delta: T,
}

#[derive(bon::Builder, Debug, Default)]
pub struct ChunkDeltaMap<T>
{
     pub deltas: rh::FxHashMap<glam::IVec3, Vec<ChunkDelta<T>>>,
}

impl<T> ChunkDeltaMap<T>
{
     pub fn new() -> Self
     where
          T: Default,
     {
          Self::default()
     }

     pub fn merge(&mut self, other: Self)
     {
          for (coord, mut deltas) in other.deltas
          {
               self.deltas.entry(coord).or_default().append(&mut deltas);
          }
     }

     pub fn insert(&mut self, coord: glam::IVec3, delta: ChunkDelta<T>)
     {
          self.deltas.entry(coord).or_default().push(delta);
     }

     pub fn get_deltas(&self, coord: glam::IVec3) -> Vec<ChunkDelta<T>>
     where
          T: Clone + Copy,
     {
          self.deltas.get(&coord).map_or(Vec::new(), |val| val.to_vec()).to_vec()
     }

     pub fn take_deltas(&mut self, coord: glam::IVec3) -> Vec<ChunkDelta<T>>
     {
          self.deltas.remove(&coord).map_or(Vec::new(), |val| val)
     }
}

pub type BlockDeltas = ChunkDeltaMap<block::Block>;

// impl DeltaValue for block::Block
// {
//      fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
//      {
//           let curr = chunk.get(delta.coord);
//           let replace = curr.max(&delta.delta);
//           *chunk.get_mut(delta.coord) = *replace
//      }
// }

pub type LightDeltas = ChunkDeltaMap<light::Light>;

// impl DeltaValue for u8
// {
//      fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
//      {
//           let _ = chunk.get_light(delta.coord);
//           todo!()
//      }
// }
