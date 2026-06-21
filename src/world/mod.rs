use std::collections;
use std::sync::mpsc;
use std::sync::{self};
use std::thread;

use crate::engine::kinematics;
use crate::engine::ray;
use crate::render::mesh;
use crate::render::util;
use crate::render::{self};
use crate::visual::atlas;
use crate::visual::mesher;

pub mod block;
pub mod chunk;
pub mod light;
pub mod terrain;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkStage
{
     #[default]
     Allocated,
     TerrainGenerated,
     DecoratorsPlaced,
     LightingPropagated,
     Meshed,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkEntry
{
     pub stage: ChunkStage,
     pub chunk: sync::Arc<chunk::Chunk>,
}

#[derive(bon::Builder, Debug, Clone)]
pub struct ChunkView
{
     pub center: glam::IVec3,
     pub chunk: sync::Arc<chunk::Chunk>,
     pub neighbors: collections::HashMap<glam::IVec3, sync::Arc<chunk::Chunk>>,
     pub size: i32,
     pub chunk_width: i32,
     pub chunk_height: i32,
}

impl ChunkView
{
     pub fn resolve(&self, world_coord: glam::IVec3) -> (glam::IVec3, glam::IVec3)
     {
          let world = self.chunk.chunk_world_coords(world_coord);
          let rel = world - self.chunk.offset();
          let chunk = self.chunk.to_chunk_coords(world_coord);

          (rel, chunk)
     }

     pub fn get_block(&self, world_position: glam::IVec3) -> block::Block
     {
          let (rel, local) = self.resolve(world_position);
          match self.neighbors.get(&rel)
          {
               | Some(chunk) => *chunk.get(local),
               | None => block::Block::EMPTY,
          }
     }
}

pub trait DeltaValue
{
     fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
     where
          Self: Sized;
}

pub type BlockDeltas = ChunkDeltaMap<block::Block>;

impl DeltaValue for block::Block
{
     fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
     where
          Self: Sized,
     {
          let curr = chunk.get(delta.coord);
          let replace = curr.max(&delta.delta);
          *chunk.get_mut(delta.coord) = *replace
     }
}

pub type LightDeltas = ChunkDeltaMap<u8>;

impl DeltaValue for u8
{
     fn resolve(&self, delta: ChunkDelta<Self>, chunk: &mut chunk::Chunk)
     where
          Self: Sized,
     {
          let _ = chunk.get_light(delta.coord);
          todo!()
     }
}

#[derive(bon::Builder, Debug, Default)]
pub struct ChunkDelta<T>
{
     pub coord: glam::IVec3,
     pub delta: T,
}

#[derive(bon::Builder, Debug, Default)]
pub struct ChunkDeltaMap<T>
{
     pub deltas: collections::HashMap<glam::IVec3, Vec<ChunkDelta<T>>>,
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
}

#[derive(bon::Builder, Debug, Default)]
pub struct ChunkMap
{
     pub chunks: sync::RwLock<collections::HashMap<glam::IVec3, ChunkEntry>>,
     pub update_times: collections::HashMap<glam::IVec3, f32>,
}

impl ChunkMap
{
     pub fn new() -> Self
     {
          Self::default()
     }

     pub fn insert(&self, coord: glam::IVec3, chunk: sync::Arc<chunk::Chunk>)
     {
          self.chunks
               .write()
               .unwrap()
               .insert(coord, ChunkEntry { stage: ChunkStage::Allocated, chunk });
     }

     pub fn remove(&self, coord: &glam::IVec3)
     {
          self.chunks.write().unwrap().remove(coord);
     }

     pub fn contains(&self, coord: &glam::IVec3) -> bool
     {
          self.chunks.read().unwrap().contains_key(coord)
     }

     pub fn set_stage(&self, coord: &glam::IVec3, stage: ChunkStage) -> bool
     {
          if let Some(chunk) = self.chunks.write().unwrap().get_mut(coord)
          {
               chunk.stage = stage;
               return true;
          }
          false
     }

     pub fn get_stage(&self, coord: &glam::IVec3) -> Option<ChunkStage>
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

     pub fn get_view(&self, target: glam::IVec3, stage_threshold: ChunkStage, size: i32)
     -> Option<ChunkView>
     {
          let map = self.chunks.read().unwrap();
          let mut neighbor_chunks = collections::HashMap::new();
          for dz in -size..=size
          {
               for dx in -size..=size
               {
                    let rel = glam::ivec3(dx, 0, dz);
                    let coord = target + rel;

                    let chunk = map.get(&coord)?;
                    if chunk.stage < stage_threshold
                    {
                         return None;
                    }

                    neighbor_chunks.insert(rel, sync::Arc::clone(&chunk.chunk));
               }
          }
          let target_chunk = sync::Arc::clone(&map[&target].chunk);
          let chunk_width = target_chunk.width() as i32;
          let chunk_height = target_chunk.height() as i32;

          Some(ChunkView {
               center: target,
               chunk: target_chunk,
               neighbors: neighbor_chunks,
               size,
               chunk_width,
               chunk_height,
          })
     }
}

#[derive(Debug, Default)]
pub enum ChunkRequest
{
     GenerateTerrain
     {
          coord: glam::IVec3,
     },
     PlaceDecorators
     {
          view: ChunkView,
     },
     PropagateLighting
     {
          view: ChunkView,
     },
     Mesh
     {
          view: ChunkView,
     },
     #[default]
     ShutdownThread,
}

#[derive(Debug)]
pub enum ChunkResponse
{
     TerrainGenerated
     {
          coord: glam::IVec3,
          chunk: sync::Arc<chunk::Chunk>,
          deltas: BlockDeltas,
     },
     DecoratorsPlaced
     {
          coord: glam::IVec3,
     },
     LightingPropagated
     {
          coord: glam::IVec3,
          chunk: sync::Arc<chunk::Chunk>,
     },
     Meshed
     {
          coord: glam::IVec3,
          raw_mesh: mesher::ChunkRawMesh,
     },
}

#[derive(bon::Builder, Debug)]
pub struct ChunkManager
{
     pub atlas: sync::Arc<atlas::TextureAtlas>,
     pub terrain: sync::Arc<terrain::TerrainGenerator>,
     pub chunk_width: usize,
     pub chunk_height: usize,
     #[builder(default = glam::IVec3::MAX)]
     pub center_chunk: glam::IVec3,

     pub view_distance: usize,

     #[builder(default)]
     pub chunk_map: ChunkMap,
     #[builder(default)]
     pub chunk_delta_map: BlockDeltas,

     #[builder(default)]
     pub pending_generated: collections::HashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_decorators: collections::HashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_lighting: collections::HashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_mesh: collections::HashSet<glam::IVec3>,

     #[builder(default)]
     pub render_chunks: collections::HashSet<glam::IVec3>,

     #[builder(default)]
     pub gfx_insert_queue: Vec<mesher::ChunkRawMesh>,
     #[builder(default)]
     pub gfx_remove_queue: Vec<glam::IVec3>,

     pub chunk_send: Option<mpsc::SyncSender<ChunkRequest>>,
     pub chunk_recv: Option<mpsc::Receiver<ChunkResponse>>,
}

impl ChunkManager
{
     pub fn spawn_worker(&mut self)
     {
          let (send_tx, send_rx) = mpsc::sync_channel(4 * self.view_distance * self.view_distance);
          let (recv_tx, recv_rx) = mpsc::channel();

          self.run_worker(send_rx, recv_tx);

          self.chunk_send = Some(send_tx);
          self.chunk_recv = Some(recv_rx);
     }

     pub fn update_chunks(&mut self, center: glam::Vec3, time: f32)
     {
          self.handle_response(time);

          self.update_good_chunks(center);

          self.cull_bad_chunks();
     }

     pub fn advance_chunk(&mut self, coord: glam::IVec3)
     {
          let stage = match self.chunk_map.get_stage(&coord)
          {
               | Some(stage) => stage,
               | None =>
               {
                    if !self.pending_generated.contains(&coord)
                    {
                         self.request_chunk_generation(coord);
                    }
                    return;
               }
          };

          match stage
          {
               | ChunkStage::Allocated | ChunkStage::TerrainGenerated =>
               {
                    if !self.pending_decorators.contains(&coord)
                    {
                         self.request_chunk_decorators(coord);
                    }
               }
               | ChunkStage::DecoratorsPlaced =>
               {
                    if !self.pending_lighting.contains(&coord)
                    {
                         self.request_chunk_lighting(coord);
                    }
               }
               | ChunkStage::LightingPropagated =>
               {
                    if !self.pending_mesh.contains(&coord)
                    {
                         self.request_chunk_meshing(coord);
                    }
               }
               | ChunkStage::Meshed =>
               {}
          }
     }

     pub fn handle_response(&mut self, time: f32)
     {
          if let Some(recv) = &self.chunk_recv
          {
               while let Ok(response) = recv.try_recv()
               {
                    match response
                    {
                         | ChunkResponse::TerrainGenerated { coord, chunk, deltas } =>
                         {
                              self.pending_generated.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_delta_map.merge(deltas);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.set_stage(&coord, ChunkStage::TerrainGenerated);
                         }
                         | ChunkResponse::DecoratorsPlaced { coord } =>
                         {
                              self.pending_decorators.remove(&coord);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.set_stage(&coord, ChunkStage::DecoratorsPlaced);
                         }
                         | ChunkResponse::LightingPropagated { coord, chunk } =>
                         {
                              self.pending_lighting.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.set_stage(&coord, ChunkStage::LightingPropagated);
                         }
                         | ChunkResponse::Meshed { coord, raw_mesh } =>
                         {
                              self.pending_mesh.remove(&coord);
                              self.chunk_map.set_stage(&coord, ChunkStage::Meshed);
                              self.chunk_map.set_time(&coord, time);
                              self.gfx_insert_queue.push(raw_mesh);
                         }
                    }
               }
          }
     }

     pub fn sync_gfx_chunks(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer)
     {
          self.gfx_insert_queue.drain(..).for_each(|raw_mesh| {
               let gfx_mesh = mesh::GfxMesh::new(context, &raw_mesh.vertices, &raw_mesh.indices);
               let name = Self::chunk_key(raw_mesh.offset);
               render.register_mesh(&name, gfx_mesh);
               render.register_resource(
                    &format!("{}_time_uni", name),
                    util::uniform::<f32>(context, "Timer uniform"),
               );
               render
                    .register_bind_group(
                         context,
                         &format!("{}_time_bg", Self::chunk_key(raw_mesh.offset)),
                         "time_layout",
                         &[&format!("{}_time_uni", name)],
                    )
                    .unwrap();
               self.render_chunks.insert(raw_mesh.offset);
          });

          self.gfx_remove_queue.drain(..).for_each(|chunk_coord| {
               let name = Self::chunk_key(chunk_coord);
               self.render_chunks.remove(&chunk_coord);
               render.unregister_mesh(&name);
               // render.unregister_resource(name);
          });
     }

     pub fn modify(&mut self, coord: glam::IVec3, block: block::Block)
     {
          let mut requests = Vec::new();
          let chunk_worldspace = self.chunk_surrounding(coord.as_vec3());
          if let Some(chunk) = self.chunk_map.chunks.write().unwrap().get_mut(&chunk_worldspace)
          {
               let coord = chunk.chunk.to_chunk_coords(coord);
               let chunk = sync::Arc::make_mut(&mut chunk.chunk);
               *chunk.get_mut(coord) = block;
               *chunk.get_light_mut(coord) = match block
               {
                    | block::Block::Air => 0,
                    | block::Block::Light => 15,
                    | _ => 0,
               };

               requests.push(chunk_worldspace);
               if coord.x == 0
               {
                    requests.push(chunk_worldspace + glam::ivec3(-1, 0, 0));
               }
               if coord.z == 0
               {
                    requests.push(chunk_worldspace + glam::ivec3(0, 0, -1));
               }
               if coord.x == chunk.width() as i32 - 1
               {
                    requests.push(chunk_worldspace + glam::ivec3(1, 0, 0));
               }
               if coord.z == chunk.width() as i32 - 1
               {
                    requests.push(chunk_worldspace + glam::ivec3(0, 0, 1));
               }
          }
          // requests.iter().for_each(|&req| self.request_chunk_meshing(req));
          requests.iter().for_each(|&req| self.request_chunk_lighting(req));
     }

     pub fn chunk_key(coord: glam::IVec3) -> String
     {
          format!("ch{}x{}x{}_mesh", coord.x, coord.y, coord.z)
     }

     pub fn chunk_surrounding(&self, center: glam::Vec3) -> glam::IVec3
     {
          glam::ivec3(
               (center.x / self.chunk_width as f32).floor() as i32,
               0,
               (center.z / self.chunk_width as f32).floor() as i32,
          )
     }

     fn chunk_in_range(&self, coord: glam::IVec3) -> bool
     {
          let rel = coord.saturating_sub(self.center_chunk);
          let rel_sq_length = (rel.x.saturating_mul(rel.x))
               .saturating_add(rel.y.saturating_mul(rel.y))
               .saturating_add(rel.z.saturating_mul(rel.z)) as usize;

          rel_sq_length < (self.view_distance * self.view_distance)
     }

     fn update_good_chunks(&mut self, center: glam::Vec3)
     {
          self.center_chunk = self.chunk_surrounding(center);
          let mut queue = collections::VecDeque::from([self.center_chunk]);
          let mut visited = collections::HashSet::from([self.center_chunk]);
          while let Some(coord) = queue.pop_back()
          {
               if !self.chunk_in_range(coord)
               {
                    continue;
               }

               self.advance_chunk(coord);

               for (dx, dz) in [(1, 0), (-1, 0), (0, -1), (0, 1)]
               {
                    let neighbor = coord + glam::ivec3(dx, 0, dz);
                    if visited.insert(neighbor)
                    {
                         queue.push_front(neighbor);
                    }
               }
          }
     }

     fn cull_bad_chunks(&mut self)
     {
          let removal = self
               .chunk_map
               .chunks
               .read()
               .unwrap()
               .keys()
               .copied()
               .filter(|&chunk_coord| !self.chunk_in_range(chunk_coord))
               .collect::<Vec<glam::IVec3>>();
          removal.iter().for_each(|&chunk_coord| {
               self.chunk_map.remove(&chunk_coord);
               self.pending_generated.remove(&chunk_coord);
               self.pending_decorators.remove(&chunk_coord);
               self.pending_lighting.remove(&chunk_coord);
               self.pending_mesh.remove(&chunk_coord);
               self.gfx_remove_queue.push(chunk_coord);
          });
     }

     fn request_chunk_generation(&mut self, coord: glam::IVec3)
     {
          if self.send_request(ChunkRequest::GenerateTerrain { coord })
          {
               self.pending_generated.insert(coord);
          }
     }

     fn request_chunk_decorators(&mut self, coord: glam::IVec3)
     {
          if let Some(view) = self.chunk_map.get_view(coord, ChunkStage::TerrainGenerated, 1)
               && self.send_request(ChunkRequest::PlaceDecorators { view })
          {
               self.pending_decorators.insert(coord);
          }
     }

     fn request_chunk_lighting(&mut self, coord: glam::IVec3)
     {
          if let Some(view) = self.chunk_map.get_view(coord, ChunkStage::DecoratorsPlaced, 1)
               && self.send_request(ChunkRequest::PropagateLighting { view })
          {
               self.pending_lighting.insert(coord);
          }
     }

     fn request_chunk_meshing(&mut self, coord: glam::IVec3)
     {
          if let Some(view) = self.chunk_map.get_view(coord, ChunkStage::LightingPropagated, 1)
               && self.send_request(ChunkRequest::Mesh { view })
          {
               self.pending_mesh.insert(coord);
          }
     }

     fn send_request(&self, request: ChunkRequest) -> bool
     {
          if let Some(send) = &self.chunk_send
          {
               return match send.try_send(request)
               {
                    | Ok(_) => true,
                    | Err(err) =>
                    {
                         log::debug!("Error sending chunk request: {}", err);
                         false
                    }
               };
          }

          log::error!("Worker thread not spawned");
          false
     }

     fn run_worker(
          &self,
          chunk_request: mpsc::Receiver<ChunkRequest>,
          chunk_response: mpsc::Sender<ChunkResponse>,
     )
     {
          let width = self.chunk_width;
          let height = self.chunk_height;
          let terrain = sync::Arc::clone(&self.terrain);
          let atlas = sync::Arc::clone(&self.atlas);

          thread::spawn(move || {
               while let Ok(request) = chunk_request.recv()
               {
                    match request
                    {
                         | ChunkRequest::GenerateTerrain { coord } =>
                         {
                              let mut chunk = chunk::Chunk::new(coord, width, height);
                              let deltas = terrain.form_chunk(&mut chunk);

                              let response = ChunkResponse::TerrainGenerated {
                                   coord,
                                   chunk: sync::Arc::new(chunk),
                                   deltas,
                              };
                              chunk_response.send(response).unwrap();
                         }
                         | ChunkRequest::PlaceDecorators { view } =>
                         {
                              let coord = view.center;

                              let response = ChunkResponse::DecoratorsPlaced { coord };
                              chunk_response.send(response).unwrap();
                         }
                         | ChunkRequest::PropagateLighting { mut view } =>
                         {
                              let coord = view.center;
                              light::ChunkLighting::new(&mut view).lighting();

                              let response = ChunkResponse::LightingPropagated {
                                   coord,
                                   chunk: sync::Arc::clone(&view.chunk),
                              };
                              chunk_response.send(response).unwrap();
                         }
                         | ChunkRequest::Mesh { view } =>
                         {
                              let coord = view.center;
                              let raw_mesh = view.chunk.raw_mesh(&atlas, &view);

                              let response = ChunkResponse::Meshed { coord, raw_mesh };
                              chunk_response.send(response).unwrap();
                         }
                         | ChunkRequest::ShutdownThread =>
                         {
                              return;
                         }
                    }
               }
          });
     }
}

impl kinematics::Collision for ChunkManager
{
     type Collider = kinematics::BoxCollider;

     fn collides(&self, collider: Self::Collider) -> bool
     {
          let center = collider.center();
          let center_chunk = self.chunk_surrounding(center);
          for dx in -1..=1
          {
               for dz in -1..=1
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

impl ray::Cast for ChunkManager
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
                              return Some(ChunkHit { block, position: idx, normal });
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
