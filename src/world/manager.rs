use std::collections;
use std::sync;
use std::sync::mpsc;
use std::thread;
use std::time;

use rustc_hash as rh;

use crate::engine::neighbors;
use crate::render;
use crate::render::mesh;
use crate::render::util;
use crate::visual::atlas;
use crate::visual::mesher;
use crate::world::block;
use crate::world::chunk;
use crate::world::delta;
use crate::world::light;
use crate::world::map;
use crate::world::terrain;
use crate::world::{self};

#[derive(Debug, Default)]
pub enum ChunkRequest
{
     GenerateTerrain
     {
          coord: glam::IVec3,
     },
     PlaceDecorators
     {
          view: world::ChunkView,
          deltas: Vec<delta::ChunkDelta<block::Block>>,
     },
     PropagateLighting
     {
          view: world::ChunkView,
     },
     Mesh
     {
          view: world::ChunkView,
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
          deltas: delta::BlockDeltas,
     },
     DecoratorsPlaced
     {
          coord: glam::IVec3,
          chunk: sync::Arc<chunk::Chunk>,
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
     pub chunk_map: map::ChunkMap,
     #[builder(default)]
     pub chunk_delta_map: delta::BlockDeltas,

     #[builder(default)]
     pub pending_generated: rh::FxHashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_decorators: rh::FxHashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_lighting: rh::FxHashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_mesh: rh::FxHashSet<glam::IVec3>,

     #[builder(default)]
     pub render_chunks: rh::FxHashSet<glam::IVec3>,

     #[builder(default)]
     pub gfx_insert_queue: Vec<mesher::ChunkRawMesh>,
     #[builder(default)]
     pub gfx_remove_queue: Vec<glam::IVec3>,

     pub chunk_send: Option<mpsc::SyncSender<ChunkRequest>>,
     pub chunk_recv: Option<mpsc::Receiver<ChunkResponse>>,
}

impl ChunkManager
{
     pub fn spawn_workers(&mut self, workers: usize)
     {
          let (send_tx, send_rx) = mpsc::sync_channel(64);
          let (recv_tx, recv_rx) = mpsc::channel();

          let shared_rx = sync::Arc::new(sync::Mutex::new(send_rx));
          (0 .. workers).for_each(|_| {
               self.run_worker(sync::Arc::clone(&shared_rx), recv_tx.clone());
          });

          self.chunk_send = Some(send_tx);
          self.chunk_recv = Some(recv_rx);
     }

     pub fn update_chunks(&mut self, center: glam::Vec3, time: f32)
     {
          self.center_chunk = self.chunk_surrounding(center);

          self.cull_bad_chunks();

          self.update_good_chunks();

          self.handle_response(time);
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
               | world::ChunkStage::Allocated | world::ChunkStage::TerrainGenerated =>
               {
                    if !self.pending_decorators.contains(&coord)
                    {
                         self.request_chunk_decorators(coord);
                    }
               }
               | world::ChunkStage::DecoratorsPlaced =>
               {
                    if !self.pending_lighting.contains(&coord)
                    {
                         self.request_chunk_lighting(coord);
                    }
               }
               | world::ChunkStage::LightingPropagated =>
               {
                    if !self.pending_mesh.contains(&coord)
                    {
                         self.request_chunk_meshing(coord);
                    }
               }
               | world::ChunkStage::Meshed =>
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
                              self.chunk_map.set_stage(&coord, world::ChunkStage::TerrainGenerated);
                         }
                         | ChunkResponse::DecoratorsPlaced { coord, chunk } =>
                         {
                              self.pending_decorators.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::DecoratorsPlaced);
                         }
                         | ChunkResponse::LightingPropagated { coord, chunk } =>
                         {
                              self.pending_lighting.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::LightingPropagated);
                         }
                         | ChunkResponse::Meshed { coord, raw_mesh } =>
                         {
                              self.pending_mesh.remove(&coord);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::Meshed);
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
               render.unregister_resource(&format!("{}_time_uni", name));
               render.unregister_bind_group(&format!("{}_time_bg", name));
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
                    | block::Block::Light => light::MAX_LIGHT,
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

     fn update_good_chunks(&mut self)
     {
          let mut queue = collections::VecDeque::from([self.center_chunk]);
          let mut visited = rh::FxHashSet::from_iter([self.center_chunk]);
          while let Some(coord) = queue.pop_back()
          {
               if !self.chunk_in_range(coord)
               {
                    continue;
               }

               self.advance_chunk(coord);

               for (dx, dz) in neighbors::von_neumann2()
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
          if let Some(view) = self.chunk_map.get_complete_view(coord, world::ChunkStage::TerrainGenerated, 1)
               && self.send_request(ChunkRequest::PlaceDecorators {
                    view,
                    deltas: self.chunk_delta_map.get_deltas(coord),
               })
          {
               self.pending_decorators.insert(coord);
          }
     }

     fn request_chunk_lighting(&mut self, coord: glam::IVec3)
     {
          if let Some(view) = self.chunk_map.get_complete_view(coord, world::ChunkStage::DecoratorsPlaced, 1)
               && self.send_request(ChunkRequest::PropagateLighting { view })
          {
               self.pending_lighting.insert(coord);
          }
     }

     fn request_chunk_meshing(&mut self, coord: glam::IVec3)
     {
          if let Some(view) =
               self.chunk_map.get_complete_view(coord, world::ChunkStage::LightingPropagated, 1)
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
          chunk_request: sync::Arc<sync::Mutex<mpsc::Receiver<ChunkRequest>>>,
          chunk_response: mpsc::Sender<ChunkResponse>,
     )
     {
          let width = self.chunk_width;
          let height = self.chunk_height;
          let terrain = sync::Arc::clone(&self.terrain);
          let atlas = sync::Arc::clone(&self.atlas);

          thread::spawn(move || {
               loop
               {
                    let request = match chunk_request.lock().unwrap().recv()
                    {
                         | Ok(request) => request,
                         | Err(err) =>
                         {
                              log::error!("Error acquiring lock on request: {}", err);
                              break;
                         }
                    };

                    let time = time::Instant::now();
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

                              log::debug!("Terrain generated: {} ms", time.elapsed().as_millis());
                         }
                         | ChunkRequest::PlaceDecorators { mut view, deltas } =>
                         {
                              let coord = view.center;
                              let chunk = sync::Arc::make_mut(&mut view.chunk);
                              deltas.into_iter().for_each(|delta| {
                                   *chunk.get_mut(delta.coord) = delta.delta;
                              });

                              let response = ChunkResponse::DecoratorsPlaced { coord, chunk: view.chunk };
                              chunk_response.send(response).unwrap();

                              log::debug!("Decorators placed: {} ms", time.elapsed().as_millis());
                         }
                         | ChunkRequest::PropagateLighting { mut view } =>
                         {
                              let coord = view.center;
                              light::ChunkLighting::new(&mut view).initial_lighting();

                              let response = ChunkResponse::LightingPropagated { coord, chunk: view.chunk };
                              chunk_response.send(response).unwrap();

                              log::debug!("Lighting propagated: {} ms", time.elapsed().as_millis());
                         }
                         | ChunkRequest::Mesh { view } =>
                         {
                              let coord = view.center;
                              let raw_mesh = view.chunk.raw_mesh(&atlas, &view);

                              let response = ChunkResponse::Meshed { coord, raw_mesh };
                              chunk_response.send(response).unwrap();

                              log::debug!("Chunk meshed: {} ms", time.elapsed().as_millis());
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
