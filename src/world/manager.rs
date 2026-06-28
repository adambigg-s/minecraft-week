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
use crate::terrain;
use crate::visual::atlas;
use crate::visual::light;
use crate::visual::mesher;
use crate::world::block;
use crate::world::chunk;
use crate::world::delta;
use crate::world::map;
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
          chunk: sync::Arc<chunk::Chunk>,
          deltas: Vec<delta::ChunkDelta<block::Block>>,
     },
     PropagateLighting
     {
          view: world::ChunkView,
          chunk: sync::Arc<chunk::Chunk>,
     },
     UpdateLighting
     {
          view: world::ChunkView,
          chunk: sync::Arc<chunk::Chunk>,
          deltas: Vec<delta::ChunkDelta<light::LightDelta>>,
     },
     Mesh
     {
          view: world::ChunkView,
          chunk: sync::Arc<chunk::Chunk>,
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
          gen_time: time::Duration,
     },
     DecoratorsPlaced
     {
          coord: glam::IVec3,
          chunk: sync::Arc<chunk::Chunk>,
          gen_time: time::Duration,
     },
     LightingPropagated
     {
          coord: glam::IVec3,
          chunk: sync::Arc<chunk::Chunk>,
          deltas: delta::LightDeltas,
          gen_time: time::Duration,
     },
     LightingUpdated
     {
          coord: glam::IVec3,
          chunk: sync::Arc<chunk::Chunk>,
          deltas: delta::LightDeltas,
          gen_time: time::Duration,
     },
     Meshed
     {
          coord: glam::IVec3,
          raw_mesh: mesher::ChunkRawMesh,
          gen_time: time::Duration,
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
     pub chunk_block_deltas: delta::BlockDeltas,
     #[builder(default)]
     pub chunk_light_deltas: delta::LightDeltas,

     #[builder(default)]
     pub pending_generated: rh::FxHashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_decorators: rh::FxHashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_lighting: rh::FxHashSet<glam::IVec3>,
     #[builder(default)]
     pub pending_lighting_updated: rh::FxHashSet<glam::IVec3>,
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

          self.resolve_chunk_deltas();

          self.cull_bad_chunks();

          self.update_good_chunks();

          self.handle_response(time);
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

               let mut light = block.emissivity().unwrap_or(light::Light::min_light());
               let removal = block == block::Block::empty();
               if removal
               {
                    light = light::Light::max_light();
               }
               self.chunk_light_deltas.insert(
                    chunk_worldspace,
                    delta::ChunkDelta {
                         coord,
                         delta: light::LightDelta {
                              light,
                              removal,
                         },
                    },
               );

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

          requests.iter().for_each(|&req| {
               self.chunk_map.try_set_stage(
                    &req,
                    world::ChunkStage::LightingPropagated,
                    world::ChunkStage::LightingPropagated,
               );
          });
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

     pub fn request_shutdown(&mut self)
     {
          (0 .. 32).for_each(|_| {
               self.send_request(ChunkRequest::ShutdownThread);
          });
          log::warn!("\n{}", self.chunk_map.telem);
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

     fn resolve_chunk_deltas(&mut self)
     {
          let mut block_delta_resolution = Vec::new();
          self.chunk_block_deltas.deltas.iter().for_each(|(&coord, deltas)| {
               if !deltas.is_empty()
               {
                    block_delta_resolution.push(coord);
               }
          });
          block_delta_resolution.into_iter().for_each(|coord| {
               self.chunk_map.try_set_stage(
                    &coord,
                    world::ChunkStage::TerrainGenerated,
                    world::ChunkStage::TerrainGenerated,
               );
          });

          let mut light_delta_resolution = Vec::new();
          self.chunk_light_deltas.deltas.iter().for_each(|(&coord, deltas)| {
               if !deltas.is_empty()
               {
                    light_delta_resolution.push(coord);
               }
          });
          light_delta_resolution.into_iter().for_each(|coord| {
               self.chunk_map.try_set_stage(
                    &coord,
                    world::ChunkStage::LightingPropagated,
                    world::ChunkStage::LightingPropagated,
               );
          });
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
          if self.send_request(ChunkRequest::GenerateTerrain {
               coord,
          })
          {
               self.pending_generated.insert(coord);
          }
     }

     fn request_chunk_decorators(&mut self, coord: glam::IVec3)
     {
          let deltas = self.chunk_block_deltas.get_deltas(coord);
          let Some(chunk) = self.chunk_map.get_chunk(&coord, world::ChunkStage::TerrainGenerated)
          else
          {
               return;
          };
          if self.send_request(ChunkRequest::PlaceDecorators {
               chunk,
               deltas,
          })
          {
               self.pending_decorators.insert(coord);
               let _ = self.chunk_block_deltas.take_deltas(coord);
          }
     }

     fn request_chunk_lighting(&mut self, coord: glam::IVec3)
     {
          let Some(view) = self.chunk_map.get_any_view(coord, world::ChunkStage::DecoratorsPlaced, 1)
          else
          {
               return;
          };
          let Some(chunk) = self.chunk_map.get_chunk(&coord, world::ChunkStage::DecoratorsPlaced)
          else
          {
               return;
          };
          if self.send_request(ChunkRequest::PropagateLighting {
               view,
               chunk,
          })
          {
               self.pending_lighting.insert(coord);
          }
     }

     fn request_chunk_light_update(&mut self, coord: glam::IVec3)
     {
          let deltas = self.chunk_light_deltas.get_deltas(coord);
          let Some(view) = self.chunk_map.get_complete_view(coord, world::ChunkStage::LightingPropagated, 1)
          else
          {
               return;
          };
          let Some(chunk) = self.chunk_map.get_chunk(&coord, world::ChunkStage::LightingPropagated)
          else
          {
               return;
          };
          if self.send_request(ChunkRequest::UpdateLighting {
               view,
               chunk,
               deltas,
          })
          {
               self.pending_lighting_updated.insert(coord);
               let _ = self.chunk_light_deltas.take_deltas(coord);
          }
     }

     fn request_chunk_meshing(&mut self, coord: glam::IVec3)
     {
          let Some(view) = self.chunk_map.get_complete_view(coord, world::ChunkStage::LightingUpdated, 1)
          else
          {
               return;
          };
          let Some(chunk) = self.chunk_map.get_chunk(&coord, world::ChunkStage::LightingUpdated)
          else
          {
               return;
          };
          if self.send_request(ChunkRequest::Mesh {
               view,
               chunk,
          })
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
                    | Err(_) => false,
               };
          }

          log::error!("Worker thread not spawned");
          false
     }

     fn advance_chunk(&mut self, coord: glam::IVec3)
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
                    if !self.pending_lighting_updated.contains(&coord)
                    {
                         self.request_chunk_light_update(coord);
                    }
               }
               | world::ChunkStage::LightingUpdated =>
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

     fn handle_response(&mut self, time: f32)
     {
          if let Some(recv) = &self.chunk_recv
          {
               while let Ok(response) = recv.try_recv()
               {
                    match response
                    {
                         | ChunkResponse::TerrainGenerated {
                              coord,
                              chunk,
                              deltas,
                              gen_time,
                         } =>
                         {
                              self.pending_generated.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::TerrainGenerated);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.telem(world::ChunkStage::TerrainGenerated, gen_time);
                              self.chunk_block_deltas.merge(&deltas);
                         }
                         | ChunkResponse::DecoratorsPlaced {
                              coord,
                              chunk,
                              gen_time,
                         } =>
                         {
                              self.pending_decorators.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::DecoratorsPlaced);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.telem(world::ChunkStage::DecoratorsPlaced, gen_time);
                         }
                         | ChunkResponse::LightingPropagated {
                              coord,
                              chunk,
                              deltas,
                              gen_time,
                         } =>
                         {
                              self.pending_lighting.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::LightingPropagated);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.telem(world::ChunkStage::LightingPropagated, gen_time);
                              self.chunk_light_deltas.merge(&deltas);
                         }
                         | ChunkResponse::LightingUpdated {
                              coord,
                              chunk,
                              deltas,
                              gen_time,
                         } =>
                         {
                              self.pending_lighting_updated.remove(&coord);
                              self.chunk_map.insert(coord, chunk);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::LightingUpdated);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.telem(world::ChunkStage::LightingUpdated, gen_time);
                              self.chunk_light_deltas.merge(&deltas);
                         }
                         | ChunkResponse::Meshed {
                              coord,
                              raw_mesh,
                              gen_time,
                         } =>
                         {
                              self.pending_mesh.remove(&coord);
                              self.chunk_map.set_stage(&coord, world::ChunkStage::Meshed);
                              self.chunk_map.set_time(&coord, time);
                              self.chunk_map.telem(world::ChunkStage::Meshed, gen_time);
                              self.gfx_insert_queue.push(raw_mesh);
                         }
                    }
               }
          }
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
                         | ChunkRequest::GenerateTerrain {
                              coord,
                         } =>
                         {
                              let mut chunk = chunk::Chunk::new(coord, width, height);
                              let deltas = terrain.form_chunk(&mut chunk);

                              let response = ChunkResponse::TerrainGenerated {
                                   coord,
                                   chunk: sync::Arc::new(chunk),
                                   deltas,
                                   gen_time: time.elapsed(),
                              };
                              chunk_response.send(response).unwrap();

                              log::debug!("Terrain generated: {} ms", time.elapsed().as_millis());
                         }
                         | ChunkRequest::PlaceDecorators {
                              mut chunk,
                              deltas,
                         } =>
                         {
                              let coord = chunk.offset();
                              let chunk_mut = sync::Arc::make_mut(&mut chunk);
                              deltas.into_iter().for_each(|delta| {
                                   *chunk_mut.get_mut(delta.coord) = delta.delta;
                              });

                              let response = ChunkResponse::DecoratorsPlaced {
                                   coord,
                                   chunk,
                                   gen_time: time.elapsed(),
                              };
                              chunk_response.send(response).unwrap();

                              log::debug!("Decorators placed: {} ms", time.elapsed().as_millis());
                         }
                         | ChunkRequest::PropagateLighting {
                              view,
                              mut chunk,
                         } =>
                         {
                              let coord = chunk.offset();
                              let deltas = light::ChunkLighting::new(&view).initialize_lighting(&mut chunk);

                              let response = ChunkResponse::LightingPropagated {
                                   coord,
                                   chunk,
                                   deltas,
                                   gen_time: time.elapsed(),
                              };
                              chunk_response.send(response).unwrap();

                              log::debug!("Lighting initially propagated: {} ms", time.elapsed().as_millis());
                         }
                         | ChunkRequest::UpdateLighting {
                              view,
                              mut chunk,
                              deltas,
                         } =>
                         {
                              let coord = chunk.offset();
                              let mut lighting = light::ChunkLighting::new(&view);
                              let deltas = lighting.update_lighting(&mut chunk, &deltas);

                              let response = ChunkResponse::LightingUpdated {
                                   coord,
                                   chunk,
                                   deltas,
                                   gen_time: time.elapsed(),
                              };
                              chunk_response.send(response).unwrap();

                              log::debug!("Lighting updated: {} ms", time.elapsed().as_millis());
                         }
                         | ChunkRequest::Mesh {
                              view,
                              chunk,
                         } =>
                         {
                              let coord = chunk.offset();
                              let raw_mesh = chunk.raw_mesh(&atlas, &view);

                              let response = ChunkResponse::Meshed {
                                   coord,
                                   raw_mesh,
                                   gen_time: time.elapsed(),
                              };
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
