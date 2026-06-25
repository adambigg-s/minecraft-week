use crate::engine::kinematics;
use crate::engine::ray;
use crate::world::block;
use crate::world::chunk;
use crate::world::manager;

impl kinematics::Collision for chunk::Chunk
{
     type Collider = kinematics::BoxCollider;

     fn collides(&self, collider: Self::Collider) -> bool
     {
          let mins = collider.lo.map(|val| val.floor() as i32);
          let maxs = collider.hi.map(|val| val.ceil() as i32);
          for z in mins[2] .. maxs[2]
          {
               for y in mins[1] .. maxs[1]
               {
                    for x in mins[0] .. maxs[0]
                    {
                         let coord = glam::ivec3(x, y, z);
                         let target_chunk = glam::ivec3(
                              (coord.x as f32 / self.width() as f32).floor() as i32,
                              0,
                              (coord.z as f32 / self.width() as f32).floor() as i32,
                         );
                         if target_chunk != self.offset()
                         {
                              continue;
                         }

                         let chunk_coord = self.to_chunk_coords(coord);
                         if !self.check_index(chunk_coord)
                         {
                              continue;
                         }
                         if self.get(chunk_coord).collides(())
                         {
                              return true;
                         }
                    }
               }
          }

          false
     }
}

impl kinematics::Collision for manager::ChunkManager
{
     type Collider = kinematics::BoxCollider;

     fn collides(&self, collider: Self::Collider) -> bool
     {
          let center = collider.center();
          let center_chunk = self.chunk_surrounding(center);
          for dx in -1 ..= 1
          {
               for dz in -1 ..= 1
               {
                    let coord = center_chunk + glam::ivec3(dx, 0, dz);
                    if let Some(entry) = self.chunk_map.chunks.read().unwrap().get(&coord)
                         && entry.chunk.collides(collider)
                    {
                         return true;
                    }
               }
          }

          false
     }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkHit
{
     pub block: block::Block,
     pub position: glam::IVec3,
     pub normal: glam::IVec3,
}

impl ray::Cast for manager::ChunkManager
{
     type Hit = ChunkHit;

     fn cast(&self, ray: ray::Ray) -> Option<Self::Hit>
     {
          let (dir, pos) = (ray.direction, ray.origin);
          let step = dir.signum().as_ivec3();
          let delta = glam::vec3(
               if dir.x != 0.0 { dir.x.recip().abs() } else { f32::INFINITY },
               if dir.y != 0.0 { dir.y.recip().abs() } else { f32::INFINITY },
               if dir.z != 0.0 { dir.z.recip().abs() } else { f32::INFINITY },
          );

          let mut idx = pos.floor().as_ivec3();
          let mut time = 0.0;
          let mut side_dist = glam::vec3(
               if dir.x > 0.0
               {
                    ((idx.x + 1) as f32 - pos.x) * delta.x
               }
               else
               {
                    (pos.x - idx.x as f32) * delta.x
               },
               if dir.y > 0.0
               {
                    ((idx.y + 1) as f32 - pos.y) * delta.y
               }
               else
               {
                    (pos.y - idx.y as f32) * delta.y
               },
               if dir.z > 0.0
               {
                    ((idx.z + 1) as f32 - pos.z) * delta.z
               }
               else
               {
                    (pos.z - idx.z as f32) * delta.z
               },
          );
          let mut normal = glam::IVec3::ZERO;

          loop
          {
               if time > ray.tspan.end
               {
                    return None;
               }

               let chunk_coords = self.chunk_surrounding(idx.as_vec3());
               if let Some(entry) = self.chunk_map.chunks.read().unwrap().get(&chunk_coords)
               {
                    let chunk = &entry.chunk;
                    let local_coord = chunk.to_chunk_coords(idx);
                    if chunk.check_index(local_coord)
                    {
                         let block = *chunk.get(local_coord);
                         if block != block::Block::Air
                         {
                              return Some(ChunkHit {
                                   block,
                                   position: idx,
                                   normal,
                              });
                         }
                    }
               }

               if side_dist.x < side_dist.y
               {
                    if side_dist.x < side_dist.z
                    {
                         time += side_dist.x;
                         side_dist.x += delta.x;
                         idx.x += step.x;
                         normal = glam::ivec3(-step.x, 0, 0);
                    }
                    else
                    {
                         time += side_dist.z;
                         side_dist.z += delta.z;
                         idx.z += step.z;
                         normal = glam::ivec3(0, 0, -step.z);
                    }
               }
               else
               {
                    if side_dist.y < side_dist.z
                    {
                         time += side_dist.y;
                         side_dist.y += delta.y;
                         idx.y += step.y;
                         normal = glam::ivec3(0, -step.y, 0);
                    }
                    else
                    {
                         time += side_dist.z;
                         side_dist.z += delta.z;
                         idx.z += step.z;
                         normal = glam::ivec3(0, 0, -step.z);
                    }
               }
          }
     }
}

impl kinematics::Collision for block::Block
{
     type Collider = ();

     fn collides(&self, _: Self::Collider) -> bool
     {
          match self
          {
               | block::Block::Air
               | block::Block::Water
               | block::Block::RedFlower
               | block::Block::BlueFlower
               | block::Block::Shrub
               | block::Block::Torch
               | block::Block::Lava => false,

               | _ => true,
          }
     }
}
