use std::{
    collections,
    ops::DerefMut,
    sync::{self, mpsc},
    thread,
};

use crate::{
    engine::{kinematics, ray, storage::buffer},
    render::{self, mesh},
    visual::{atlas, mesher},
};

pub mod block;
pub mod chunk;
pub mod terrain;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkStage {
    #[default]
    Allocated,
    TerrainGenerated,
    DecoratorsPlaced,
    LightingPropagated,
    Meshed,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkEntry {
    pub stage: ChunkStage,
    pub chunk: sync::Arc<chunk::Chunk>,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkView {
    pub target: glam::IVec3,
    pub target_chunk: sync::Arc<chunk::Chunk>,
    pub neighbor_chunks: collections::HashMap<glam::IVec3, sync::Arc<chunk::Chunk>>,
    pub size: i32,
    pub chunk_width: i32,
    pub chunk_height: i32,
}

impl ChunkView {
    pub fn resolve(&self, world_position: glam::IVec3) -> (glam::IVec3, glam::IVec3) {
        let chunk_worldspace = glam::ivec3(
            world_position.x.div_euclid(self.chunk_width),
            world_position.y.div_euclid(self.chunk_height),
            world_position.z.div_euclid(self.chunk_width),
        );
        let rel = chunk_worldspace - self.target;
        let local = self.target_chunk.to_chunk_coords(world_position);

        (rel, local)
    }

    pub fn get_block(&self, world_position: glam::IVec3) -> block::Block {
        let (rel, local) = self.resolve(world_position);
        match self.neighbor_chunks.get(&rel) {
            | Some(chunk) => *chunk.get(local),
            | None => block::Block::EMPTY,
        }
    }
}

// #[derive(bon::Builder, Debug)]
// pub struct ChunkRegion {
//     pub target: glam::IVec3,
//     pub target_chunk: sync::Arc<sync::RwLock<chunk::Chunk>>,
//     pub neighbor_chunks: collections::HashMap<glam::IVec3, sync::Arc<sync::RwLock<chunk::Chunk>>>,
//     pub size: i32,
//     pub chunk_width: i32,
//     pub chunk_height: i32,
// }

// impl ChunkRegion {
//     pub fn resolve(&self, world_position: glam::IVec3) -> (glam::IVec3, glam::IVec3) {
//         let chunk_worldspace = glam::ivec3(
//             world_position.x.div_euclid(self.chunk_width),
//             world_position.y.div_euclid(self.chunk_height),
//             world_position.z.div_euclid(self.chunk_width),
//         );
//         let rel = chunk_worldspace - self.target;
//         let local = self.read_target().to_chunk_coords(world_position);

//         (rel, local)
//     }

//     pub fn get_block(&self, world_position: glam::IVec3) -> block::Block {
//         let (rel, local) = self.resolve(world_position);
//         match self.neighbor_chunks.get(&rel) {
//             | Some(chunk) => *chunk.read().unwrap().get(local),
//             | None => block::Block::EMPTY,
//         }
//     }

//     pub fn set_block(&self, world_position: glam::IVec3, block: block::Block) {
//         let (rel, local) = self.resolve(world_position);
//         match self.neighbor_chunks.get(&rel) {
//             | Some(chunk) => *chunk.write().unwrap().get_mut(local) = block,
//             | None => {}
//         }
//     }

//     pub fn get_target_block(&self, coord: glam::IVec3) -> block::Block {
//         *self.read_target().get(coord)
//     }

//     pub fn set_target_block(&self, coord: glam::IVec3, block: block::Block) {
//         *self.write_target().get_mut(coord) = block;
//     }

//     pub fn read_target(&self) -> sync::RwLockReadGuard<'_, chunk::Chunk> {
//         self.neighbor_chunks[&glam::IVec3::ZERO].read().unwrap()
//     }

//     pub fn write_target(&self) -> sync::RwLockWriteGuard<'_, chunk::Chunk> {
//         self.neighbor_chunks[&glam::IVec3::ZERO].write().unwrap()
//     }
// }

#[derive(bon::Builder, Debug, Default)]
pub struct ChunkMap {
    chunks: sync::RwLock<collections::HashMap<glam::IVec3, ChunkEntry>>,
}

impl ChunkMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, coord: glam::IVec3, chunk: chunk::Chunk) {
        self.chunks.write().unwrap().insert(
            coord,
            ChunkEntry {
                stage: ChunkStage::Allocated,
                chunk: sync::Arc::new(chunk),
            },
        );
    }

    pub fn remove(&self, coord: &glam::IVec3) {
        self.chunks.write().unwrap().remove(coord);
    }

    pub fn contains(&self, coord: &glam::IVec3) -> bool {
        self.chunks.read().unwrap().contains_key(coord)
    }

    pub fn set_stage(&self, coord: &glam::IVec3, stage: ChunkStage) -> bool {
        if let Some(chunk) = self.chunks.write().unwrap().get_mut(coord) {
            chunk.stage = stage;
            return true;
        }
        false
    }

    pub fn get_stage(&self, coord: &glam::IVec3) -> Option<ChunkStage> {
        if let Some(chunk) = self.chunks.read().unwrap().get(coord) {
            return Some(chunk.stage);
        }
        None
    }

    pub fn get_view(&self, target: glam::IVec3, stage_threshold: ChunkStage, size: i32) -> Option<ChunkView> {
        let map = self.chunks.read().unwrap();
        let mut neighbor_chunks = collections::HashMap::new();

        for dz in -size..=size {
            for dx in -size..=size {
                let rel = glam::ivec3(dx, 0, dz);
                let coord = target + rel;

                let chunk = map.get(&coord)?;
                if chunk.stage < stage_threshold {
                    return None;
                }

                neighbor_chunks.insert(rel, sync::Arc::clone(&chunk.chunk));
            }
        }
        let target_chunk = sync::Arc::clone(&map[&target].chunk);
        let chunk_width = target_chunk.width() as i32;
        let chunk_height = target_chunk.height() as i32;

        Some(ChunkView {
            target,
            target_chunk,
            neighbor_chunks,
            size,
            chunk_width,
            chunk_height,
        })
    }

    // pub fn get_region(
    //     &self,
    //     target: glam::IVec3,
    //     stage_threshold: ChunkStage,
    //     size: i32,
    //     chunk_width: i32,
    //     chunk_height: i32,
    // ) -> Option<ChunkRegion> {
    //     // let map = self.chunks.read().unwrap();
    //     let map = &self.chunks;
    //     let mut neighbor_chunks = collections::HashMap::new();

    //     for dz in -size..=size {
    //         for dx in -size..=size {
    //             let rel = glam::ivec3(dx, 0, dz);
    //             let coord = target + glam::ivec3(dx, 0, dz);

    //             let chunk = map.get(&coord)?;
    //             if chunk.read().unwrap().get_stage() < stage_threshold {
    //                 return None;
    //             }

    //             neighbor_chunks.insert(rel, sync::Arc::clone(&chunk));
    //         }
    //     }
    //     let target_chunk = sync::Arc::clone(&map.get(&target).unwrap());

    //     Some(ChunkRegion {
    //         target,
    //         target_chunk,
    //         neighbor_chunks,
    //         size,
    //         chunk_width,
    //         chunk_height,
    //     })
    // }
}

#[derive(Debug, Default)]
pub enum ChunkRequest {
    GenerateTerrain {
        coord: glam::IVec3,
    },
    PlaceDecorators {
        view: ChunkView,
    },
    PropagateLighting {
        view: ChunkView,
    },
    Mesh {
        view: ChunkView,
    },
    #[default]
    ShutdownThread,
}

#[derive(Debug)]
pub enum ChunkResponse {
    TerrainGenerated { coord: glam::IVec3, chunk: chunk::Chunk },
    DecoratorsPlaced { coord: glam::IVec3 },
    LightingPropagated { coord: glam::IVec3 },
    Meshed { coord: glam::IVec3, raw_mesh: mesher::ChunkRawMesh },
}

#[derive(bon::Builder, Debug)]
pub struct ChunkManager {
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

impl ChunkManager {
    pub fn spawn_worker(&mut self) {
        let (send_tx, send_rx) = mpsc::sync_channel(4 * self.view_distance);
        let (recv_tx, recv_rx) = mpsc::channel();

        self.run_worker(send_rx, recv_tx);

        self.chunk_send = Some(send_tx);
        self.chunk_recv = Some(recv_rx);
    }

    pub fn update_chunks(&mut self, center: glam::Vec3) {
        self.handle_response();

        self.center_chunk = self.chunk_surrounding(center);
        let mut queue = collections::VecDeque::from([self.center_chunk]);
        let mut visited = collections::HashSet::from([self.center_chunk]);
        while let Some(coord) = queue.pop_back() {
            if !self.chunk_in_range(coord) {
                continue;
            }

            self.advance_chunk(coord);

            for (dx, dz) in [(1, 0), (-1, 0), (0, -1), (0, 1)] {
                let neighbor = coord + glam::ivec3(dx, 0, dz);
                if visited.insert(neighbor) {
                    queue.push_front(neighbor);
                }
            }
        }
    }

    pub fn advance_chunk(&mut self, coord: glam::IVec3) {
        let stage = match self.chunk_map.get_stage(&coord) {
            | Some(stage) => stage,
            | None => {
                if !self.pending_generated.contains(&coord) {
                    self.request_chunk_generation(coord);
                }
                return;
            }
        };

        match stage {
            | ChunkStage::Allocated | ChunkStage::TerrainGenerated => {
                if !self.pending_decorators.contains(&coord) {
                    self.request_chunk_decorators(coord);
                }
            }
            | ChunkStage::DecoratorsPlaced => {
                if !self.pending_lighting.contains(&coord) {
                    self.request_chunk_lighting(coord);
                }
            }
            | ChunkStage::LightingPropagated => {
                if !self.pending_mesh.contains(&coord) {
                    self.request_chunk_meshing(coord);
                }
            }
            | ChunkStage::Meshed => {}
        }
    }

    pub fn handle_response(&mut self) {
        if let Some(recv) = &self.chunk_recv {
            while let Ok(response) = recv.try_recv() {
                match response {
                    | ChunkResponse::TerrainGenerated { coord, chunk } => {
                        self.pending_generated.remove(&coord);
                        self.chunk_map.insert(coord, chunk);
                        self.chunk_map.set_stage(&coord, ChunkStage::TerrainGenerated);
                    }
                    | ChunkResponse::DecoratorsPlaced { coord } => {
                        self.pending_decorators.remove(&coord);
                        self.chunk_map.set_stage(&coord, ChunkStage::DecoratorsPlaced);
                    }
                    | ChunkResponse::LightingPropagated { coord } => {
                        self.pending_lighting.remove(&coord);
                        self.chunk_map.set_stage(&coord, ChunkStage::LightingPropagated);
                    }
                    | ChunkResponse::Meshed { coord, raw_mesh } => {
                        self.pending_mesh.remove(&coord);
                        self.chunk_map.set_stage(&coord, ChunkStage::Meshed);
                        self.gfx_insert_queue.push(raw_mesh);
                    }
                }
            }
        }
    }

    pub fn sync_gfx_chunks(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer) {
        self.gfx_insert_queue.drain(..).for_each(|raw_mesh| {
            let gfx_mesh = mesh::GfxMesh::new(context, &raw_mesh.vertices, &raw_mesh.indices);
            render.register_mesh(&Self::chunk_key(raw_mesh.offset), gfx_mesh);
            self.render_chunks.insert(raw_mesh.offset);
        });
    }

    pub fn modify(&mut self, coord: glam::IVec3, block: block::Block) {}

    pub fn chunk_key(coord: glam::IVec3) -> String {
        format!("ch{}x{}x{}_mesh", coord.x, coord.y, coord.z)
    }

    pub fn chunk_surrounding(&self, center: glam::Vec3) -> glam::IVec3 {
        glam::ivec3(
            (center.x / self.chunk_width as f32).floor() as i32,
            0,
            (center.z / self.chunk_width as f32).floor() as i32,
        )
    }

    fn chunk_in_range(&self, coord: glam::IVec3) -> bool {
        let rel = coord.saturating_sub(self.center_chunk);
        let rel_sq_length = (rel.x.saturating_mul(rel.x))
            .saturating_add(rel.y.saturating_mul(rel.y))
            .saturating_add(rel.z.saturating_mul(rel.z)) as usize;

        rel_sq_length < (self.view_distance * self.view_distance)
    }

    fn request_chunk_generation(&mut self, coord: glam::IVec3) {
        if self.send_request(ChunkRequest::GenerateTerrain { coord }) {
            self.pending_generated.insert(coord);
        }
    }

    fn request_chunk_decorators(&mut self, coord: glam::IVec3) {
        if let Some(view) = self.chunk_map.get_view(coord, ChunkStage::TerrainGenerated, 1)
            && self.send_request(ChunkRequest::PlaceDecorators { view })
        {
            self.pending_decorators.insert(coord);
        }
    }

    fn request_chunk_lighting(&mut self, coord: glam::IVec3) {
        if let Some(view) = self.chunk_map.get_view(coord, ChunkStage::DecoratorsPlaced, 1)
            && self.send_request(ChunkRequest::PropagateLighting { view })
        {
            self.pending_lighting.insert(coord);
        }
    }

    fn request_chunk_meshing(&mut self, coord: glam::IVec3) {
        if let Some(view) = self.chunk_map.get_view(coord, ChunkStage::LightingPropagated, 1)
            && self.send_request(ChunkRequest::PropagateLighting { view })
        {
            self.pending_mesh.insert(coord);
        }
    }

    fn send_request(&self, request: ChunkRequest) -> bool {
        if let Some(send) = &self.chunk_send {
            return match send.try_send(request) {
                | Ok(_) => true,
                | Err(err) => {
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
    ) {
        let width = self.chunk_width;
        let height = self.chunk_height;
        let terrain = sync::Arc::clone(&self.terrain);
        let atlas = sync::Arc::clone(&self.atlas);

        thread::spawn(move || {
            while let Ok(request) = chunk_request.recv() {
                match request {
                    | ChunkRequest::GenerateTerrain { coord } => {
                        let mut chunk = chunk::Chunk::new(coord, width, height);
                        terrain.form_chunk(&mut chunk);

                        let response = ChunkResponse::TerrainGenerated { coord, chunk };
                        chunk_response.send(response).unwrap();
                    }
                    | ChunkRequest::PlaceDecorators { view } => {
                        let coord = view.target;

                        let response = ChunkResponse::DecoratorsPlaced { coord };
                        chunk_response.send(response).unwrap();
                    }
                    | ChunkRequest::PropagateLighting { view } => {
                        let coord = view.target;

                        let response = ChunkResponse::LightingPropagated { coord };
                        chunk_response.send(response).unwrap();
                    }
                    | ChunkRequest::Mesh { view } => {
                        let coord = view.target;
                        let raw_mesh = view.target_chunk.raw_mesh(&atlas, &view);

                        log::warn!("Chunk meshed ");

                        let response = ChunkResponse::Meshed { coord, raw_mesh };
                        chunk_response.send(response).unwrap();
                    }
                    | ChunkRequest::ShutdownThread => {
                        return;
                    }
                }
            }
        });
    }
}

// impl ChunkManager {
//     pub fn spawn_worker(&mut self) {
//         let (send_tx, send_rx) = mpsc::sync_channel(4 * self.view_distance);
//         let (recv_tx, recv_rx) = mpsc::channel();

//         self.run_worker(send_rx, recv_tx);

//         self.chunk_send = Some(send_tx);
//         self.chunk_recv = Some(recv_rx);
//     }

//     pub fn update_chunks(&mut self, center: glam::Vec3) {
//         self.handle_response();

//         self.center_chunk = self.chunk_surrounding(center);
//         let mut queue = collections::VecDeque::from([self.center_chunk]);
//         let mut visited = collections::HashSet::from([self.center_chunk]);
//         while let Some(coord) = queue.pop_back() {
//             if !self.chunk_in_range(coord) {
//                 continue;
//             }

//             self.advance_chunk(coord);

//             for (dx, dz) in [(1, 0), (-1, 0), (0, -1), (0, 1)] {
//                 let neighbor = coord + glam::ivec3(dx, 0, dz);
//                 if visited.insert(neighbor) {
//                     queue.push_front(neighbor);
//                 }
//             }
//         }

//         // let removals = self
//         //     .chunks
//         //     .chunks
//         //     .keys()
//         //     .copied()
//         //     .filter(|&key| !self.chunk_in_range(key))
//         //     .collect::<Vec<glam::IVec3>>();
//         // let removals = self
//         //     .chunks
//         //     .chunks
//         //     .read()
//         //     .unwrap()
//         //     .keys()
//         //     .copied()
//         //     .filter(|&key| !self.chunk_in_range(key))
//         //     .collect::<Vec<glam::IVec3>>();
//         // for coord in removals {
//         //     self.chunks.remove(&coord);
//         //     self.pending_generated.remove(&coord);
//         //     self.pending_decorators.remove(&coord);
//         //     self.pending_lighting.remove(&coord);
//         //     self.pending_mesh.remove(&coord);
//         //     self.render_chunks.remove(&coord);
//         //     self.gfx_remove_queue.push(coord);
//         // }
//     }

//     pub fn advance_chunk(&mut self, coord: glam::IVec3) {
//         let stage = match self.chunks.get_stage(coord) {
//             | Some(stage) => stage,
//             | None => {
//                 if !self.pending_generated.contains(&coord) {
//                     self.request_chunk_generation(coord);
//                 }
//                 return;
//             }
//         };

//         match stage {
//             | ChunkStage::Allocated | ChunkStage::TerrainGenerated => {
//                 if !self.pending_decorators.contains(&coord) {
//                     self.request_chunk_decorators(coord);
//                 }
//             }
//             | ChunkStage::DecoratorsPlaced => {
//                 if !self.pending_lighting.contains(&coord) {
//                     self.request_chunk_lighting(coord);
//                 }
//             }
//             | ChunkStage::LightingPropagated => {
//                 if !self.pending_mesh.contains(&coord) && !self.render_chunks.contains(&coord) {
//                     self.request_chunk_meshing(coord);
//                 }
//             }
//             | ChunkStage::Meshed => {}
//         }
//     }

//     pub fn handle_response(&mut self) {
//         if let Some(recv) = &self.chunk_recv {
//             let mut count = 0;
//             while let Ok(response) = recv.try_recv() {
//                 count += 1;

//                 match response {
//                     | ChunkResponse::TerrainGenerated { coord, chunk } => {
//                         self.pending_generated.remove(&coord);
//                         if self.chunk_in_range(coord) {
//                             self.chunks.insert(coord, chunk);
//                             self.chunks.set_stage(coord, ChunkStage::TerrainGenerated);
//                         }
//                     }
//                     | ChunkResponse::DecoratorsPlaced { coord } => {
//                         self.pending_decorators.remove(&coord);
//                         self.chunks.set_stage(coord, ChunkStage::DecoratorsPlaced);
//                     }
//                     | ChunkResponse::LightingPropagated { coord } => {
//                         self.pending_lighting.remove(&coord);
//                         self.chunks.set_stage(coord, ChunkStage::LightingPropagated);
//                     }
//                     | ChunkResponse::Meshed { coord, raw_mesh } => {
//                         self.pending_mesh.remove(&coord);
//                         self.chunks.set_stage(coord, ChunkStage::Meshed);
//                         if self.chunks.contains(&coord) {
//                             self.gfx_insert_queue.push(raw_mesh);
//                         }
//                     }
//                 }
//             }
//             if count > 0 {
//                 log::debug!("{} chunk responses processed", count);
//             }
//         }
//     }

//     pub fn sync_gfx_chunks(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer) {
//         self.gfx_insert_queue.drain(..).for_each(|raw_mesh| {
//             let gfx_mesh = mesh::GfxMesh::new(context, &raw_mesh.vertices, &raw_mesh.indices);
//             render.register_mesh(&Self::chunk_key(raw_mesh.offset), gfx_mesh);
//             self.render_chunks.insert(raw_mesh.offset);
//         });
//     }

//     pub fn modify(&mut self, coord: glam::IVec3, block: block::Block) {}

//     pub fn chunk_key(coord: glam::IVec3) -> String {
//         format!("ch{}x{}x{}_mesh", coord.x, coord.y, coord.z)
//     }

//     pub fn chunk_surrounding(&self, center: glam::Vec3) -> glam::IVec3 {
//         glam::ivec3(
//             (center.x / self.chunk_width as f32).floor() as i32,
//             0,
//             (center.z / self.chunk_width as f32).floor() as i32,
//         )
//     }

//     fn chunk_in_range(&self, coord: glam::IVec3) -> bool {
//         let rel = coord.saturating_sub(self.center_chunk);
//         let rel_sq_length = (rel.x.saturating_mul(rel.x))
//             .saturating_add(rel.y.saturating_mul(rel.y))
//             .saturating_add(rel.z.saturating_mul(rel.z)) as usize;

//         rel_sq_length < (self.view_distance * self.view_distance)
//     }

//     fn send_request(&self, request: ChunkRequest) -> bool {
//         if let Some(send) = &self.chunk_send {
//             return match send.try_send(request) {
//                 | Ok(_) => true,
//                 | Err(err) => {
//                     log::debug!("Error sending chunk request: {}", err);
//                     false
//                 }
//             };
//         }

//         log::error!("Worker thread not spawned");
//         false
//     }

//     fn request_chunk_generation(&mut self, coord: glam::IVec3) {
//         if self.send_request(ChunkRequest::GenerateTerrain { coord }) {
//             self.pending_generated.insert(coord);
//         }
//     }

//     fn request_chunk_decorators(&mut self, coord: glam::IVec3) {
//         if let Some(region) = self.chunks.get_region(
//             coord,
//             ChunkStage::TerrainGenerated,
//             1,
//             self.chunk_width as i32,
//             self.chunk_height as i32,
//         ) && self.send_request(ChunkRequest::PlaceDecorators { region })
//         {
//             self.pending_decorators.insert(coord);
//         }
//     }

//     fn request_chunk_lighting(&mut self, coord: glam::IVec3) {
//         if let Some(region) = self.chunks.get_region(
//             coord,
//             ChunkStage::DecoratorsPlaced,
//             1,
//             self.chunk_width as i32,
//             self.chunk_height as i32,
//         ) && self.send_request(ChunkRequest::PropagateLighting { region })
//         {
//             self.pending_lighting.insert(coord);
//         }
//     }

//     fn request_chunk_meshing(&mut self, coord: glam::IVec3) {
//         if let Some(region) = self.chunks.get_region(
//             coord,
//             ChunkStage::LightingPropagated,
//             1,
//             self.chunk_width as i32,
//             self.chunk_height as i32,
//         ) && self.send_request(ChunkRequest::Mesh { region })
//         {
//             self.pending_mesh.insert(coord);
//         }
//     }

//     fn run_worker(
//         &self,
//         chunk_request: mpsc::Receiver<ChunkRequest>,
//         chunk_response: mpsc::Sender<ChunkResponse>,
//     ) {
//         let width = self.chunk_width;
//         let height = self.chunk_height;
//         let terrain = sync::Arc::clone(&self.terrain);
//         let atlas = sync::Arc::clone(&self.atlas);

//         thread::spawn(move || {
//             while let Ok(request) = chunk_request.recv() {
//                 match request {
//                     | ChunkRequest::GenerateTerrain { coord } => {
//                         let mut chunk = chunk::Chunk::new(coord, width, height);
//                         terrain.form_chunk(&mut chunk);
//                         *chunk.get_stage_mut() = ChunkStage::TerrainGenerated;

//                         let response = ChunkResponse::TerrainGenerated { coord, chunk };
//                         chunk_response.send(response).unwrap();
//                     }
//                     | ChunkRequest::PlaceDecorators { region } => {
//                         let coord = region.target;
//                         let response = ChunkResponse::DecoratorsPlaced { coord };
//                         chunk_response.send(response).unwrap();
//                     }
//                     | ChunkRequest::PropagateLighting { region } => {
//                         let coord = region.target;
//                         let response = ChunkResponse::LightingPropagated { coord };
//                         chunk_response.send(response).unwrap();
//                     }
//                     | ChunkRequest::Mesh { region } => {
//                         let coord = region.target;
//                         let raw_mesh = region.read_target().raw_mesh(&atlas, &region);

//                         let response = ChunkResponse::Meshed { coord, raw_mesh };
//                         chunk_response.send(response).unwrap();
//                     }
//                     | ChunkRequest::ShutdownThread => {
//                         return;
//                     }
//                 }
//             }
//         });
//     }
// }

impl kinematics::Collision for ChunkManager {
    type Collider = kinematics::BoxCollider;

    fn collides(&self, collider: Self::Collider) -> bool {
        // let center = collider.center();
        // let center_chunk = self.chunk_surrounding(center);
        // for dx in -1..=1 {
        //     for dz in -1..=1 {
        //         let coord = center_chunk + glam::ivec3(dx, 0, dz);
        //         if let Some(chunk) = self.chunks.get(&coord)
        //             && chunk.collides(collider)
        //         {
        //             return true;
        //         }
        //     }
        // }

        // false
        todo!()
    }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkHit {
    pub block: block::Block,
    pub position: glam::IVec3,
    pub normal: glam::IVec3,
}

impl ray::Cast for ChunkManager {
    type Hit = ChunkHit;

    fn cast(&self, ray: ray::Ray) -> Option<Self::Hit> {
        // let (dir, pos) = (ray.direction, ray.origin);
        // let step = dir.signum().as_ivec3();
        // let delta = glam::vec3(
        //     if dir.x != 0.0 { dir.x.recip().abs() } else { f32::INFINITY },
        //     if dir.y != 0.0 { dir.y.recip().abs() } else { f32::INFINITY },
        //     if dir.z != 0.0 { dir.z.recip().abs() } else { f32::INFINITY },
        // );

        // let mut idx = pos.floor().as_ivec3();
        // let mut time = 0.0;
        // #[rustfmt::skip]
        // let mut side_dist = glam::vec3(
        //     if dir.x > 0.0 { ((idx.x + 1) as f32 - pos.x) * delta.x } else { (pos.x - idx.x as f32) * delta.x },
        //     if dir.y > 0.0 { ((idx.y + 1) as f32 - pos.y) * delta.y } else { (pos.y - idx.y as f32) * delta.y },
        //     if dir.z > 0.0 { ((idx.z + 1) as f32 - pos.z) * delta.z } else { (pos.z - idx.z as f32) * delta.z },
        // );
        // let mut normal = glam::IVec3::ZERO;

        // loop {
        //     if time > ray.tspan.end {
        //         return None;
        //     }

        //     let chunk_coords = self.chunk_surrounding(idx.as_vec3());
        //     if let Some(chunk) = self.chunks.get(&chunk_coords) {
        //         let local_coord = chunk.to_chunk_coords(idx);
        //         if chunk.check_index(local_coord) {
        //             let block = *chunk.get(local_coord);
        //             if block != block::Block::Air {
        //                 return Some(ChunkHit { block, position: idx, normal });
        //             }
        //         }
        //     }

        //     if side_dist.x < side_dist.y {
        //         if side_dist.x < side_dist.z {
        //             time += side_dist.x;
        //             side_dist.x += delta.x;
        //             idx.x += step.x;
        //             normal = glam::ivec3(-step.x, 0, 0);
        //         }
        //         else {
        //             time += side_dist.z;
        //             side_dist.z += delta.z;
        //             idx.z += step.z;
        //             normal = glam::ivec3(0, 0, -step.z);
        //         }
        //     }
        //     else {
        //         if side_dist.y < side_dist.z {
        //             time += side_dist.y;
        //             side_dist.y += delta.y;
        //             idx.y += step.y;
        //             normal = glam::ivec3(0, -step.y, 0);
        //         }
        //         else {
        //             time += side_dist.z;
        //             side_dist.z += delta.z;
        //             idx.z += step.z;
        //             normal = glam::ivec3(0, 0, -step.z);
        //         }
        //     }
        // }
        todo!()
    }
}
