use std::{
    collections,
    sync::{self, mpsc},
    thread,
};

use crate::{
    atlas, block,
    engine::storage::buffer,
    mesher,
    render::{self, mesh},
    terrain::{self},
};

#[derive(bon::Builder, Debug)]
pub struct Chunk {
    pub blocks: buffer::Buffer<block::Block, 3>,
    pub offset: glam::IVec3,
    pub height: usize,
    pub width: usize,
}

impl Chunk {
    pub fn new(offset: glam::IVec3, width: usize, height: usize) -> Self {
        let blocks = buffer::Buffer::new_zeroed([width, height, width]);

        Self { blocks, offset, height, width }
    }

    pub fn size(&self) -> glam::IVec3 {
        glam::ivec3(self.width as i32, self.height as i32, self.width as i32)
    }

    pub fn to_index(&self, coord: glam::IVec3) -> [usize; 3] {
        coord.to_array().map(|ele| ele as usize)
    }

    pub fn check_index(&self, coord: glam::IVec3) -> bool {
        let index = self.to_index(coord);
        self.blocks.surrounds(index)
    }

    pub fn get(&self, coord: glam::IVec3) -> &block::Block {
        self.blocks.get(self.to_index(coord))
    }

    pub fn get_mut(&mut self, coord: glam::IVec3) -> &mut block::Block {
        self.blocks.get_mut(self.to_index(coord))
    }

    pub fn to_chunk_coords(&self, coord: glam::IVec3) -> glam::IVec3 {
        coord.rem_euclid(self.size())
    }

    pub fn raw_mesh(
        &self,
        atlas: &atlas::TextureAtlas,
        assistant: mesher::ChunkMeshingAssisant,
    ) -> ChunkRawMesh {
        let mesher = mesher::ChunkMesher { chunks: assistant, atlas };
        let mut rectilinear = mesher.to_rectilinear();
        mesher.map_uvs(&mut rectilinear);

        let mut vertices = Vec::new();
        (0..rectilinear.size).for_each(|index| {
            let mesher::RectilinearMeshSlice { pos, nor, uvs, .. } = rectilinear.quad_slice(index);

            (0..4).for_each(|vertex| {
                vertices.push(mesher::TerrainVertex { pos: pos[vertex], nor: nor[vertex], tex: uvs[vertex] });
            });
        });
        let indices = rectilinear.indices;
        let offset = self.offset;

        ChunkRawMesh { vertices, indices, offset }
    }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkRawMesh {
    pub vertices: Vec<mesher::TerrainVertex>,
    pub indices: Vec<u32>,
    pub offset: glam::IVec3,
}

#[derive(Debug, Default)]
pub enum ChunkRequest {
    Generate {
        coord: glam::IVec3,
    },
    Mesh {
        chunk: sync::Arc<Chunk>,
        assistant: mesher::ChunkMeshingAssisant,
    },
    #[default]
    Cleanup,
}

#[derive(Debug)]
pub enum ChunkResponse {
    Generated { coord: glam::IVec3, chunk: Chunk },
    Meshed { coord: glam::IVec3, raw_mesh: ChunkRawMesh },
}

#[derive(bon::Builder, Debug)]
pub struct ChunkManager {
    pub atlas: sync::Arc<atlas::TextureAtlas>,
    pub terrain: sync::Arc<terrain::TerrainGenerator>,
    pub view_distance: usize,
    pub chunk_width: usize,
    pub chunk_height: usize,
    #[builder(default = glam::IVec3::MAX)]
    pub center_chunk: glam::IVec3,

    #[builder(default)]
    pub chunks: collections::HashMap<glam::IVec3, sync::Arc<Chunk>>,
    #[builder(default)]
    pub pending_chunks: collections::HashSet<glam::IVec3>,
    #[builder(default)]
    pub render_chunks: collections::HashSet<glam::IVec3>,
    #[builder(default)]
    pub pending_render_chunks: collections::HashSet<glam::IVec3>,

    #[builder(default)]
    pub gfx_insert_queue: Vec<ChunkRawMesh>,
    #[builder(default)]
    pub gfx_remove_queue: Vec<glam::IVec3>,

    pub chunk_send: Option<mpsc::SyncSender<ChunkRequest>>,
    pub chunk_recv: Option<mpsc::Receiver<ChunkResponse>>,
}

impl ChunkManager {
    pub fn spawn_worker(&mut self) {
        let (send_tx, send_rx) = mpsc::sync_channel(4 * self.view_distance * self.view_distance);
        let (recv_tx, recv_rx) = mpsc::channel();

        self.async_chunk_response(send_rx, recv_tx);

        self.chunk_send = Some(send_tx);
        self.chunk_recv = Some(recv_rx);
    }

    pub fn update_chunks(&mut self, center: glam::Vec3) {
        if let Some(recv) = &self.chunk_recv {
            let mut count = 0;
            while let Ok(response) = recv.try_recv() {
                count += 1;
                match response {
                    | ChunkResponse::Generated { coord, chunk } => {
                        if self.chunk_in_range(coord) {
                            self.chunks.insert(coord, sync::Arc::new(chunk));
                        }
                        self.pending_chunks.remove(&coord);
                    }
                    | ChunkResponse::Meshed { coord, raw_mesh } => {
                        if self.chunks.contains_key(&coord) {
                            self.gfx_insert_queue.push(raw_mesh);
                        }
                    }
                }
            }
            if count > 0 {
                log::debug!("{} requests fulfilled", count);
            }
        }

        self.center_chunk = self.chunk_surrounding(center);
        let range = self.view_distance as i32;
        for dz in -range..=range {
            for dx in -range..=range {
                let coord = self.center_chunk + glam::ivec3(dx, 0, dz);
                if !self.chunk_in_range(coord) {
                    continue;
                }

                if !self.pending_chunks.contains(&coord) && !self.chunks.contains_key(&coord) {
                    self.request_chunk_generation(coord);
                }

                if !self.render_chunks.contains(&coord)
                    && !self.pending_render_chunks.contains(&coord)
                    && self.chunks.contains_key(&coord)
                {
                    self.request_chunk_meshing(coord);
                }
            }
        }

        let removal = self
            .chunks
            .keys()
            .copied()
            .filter(|&coord| !self.chunk_in_range(coord))
            .collect::<Vec<glam::IVec3>>();
        removal.into_iter().for_each(|coord| {
            self.chunks.remove(&coord);
            self.pending_render_chunks.remove(&coord);
            self.gfx_remove_queue.push(coord);
        });
    }

    pub fn sync_gfx_chunks(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer) {
        self.gfx_insert_queue
            .drain(..)
            .filter(|raw_mesh| self.chunks.contains_key(&raw_mesh.offset))
            .for_each(|raw_mesh| {
                let gfx_mesh = mesh::GfxMesh::new(context, &raw_mesh.vertices, &raw_mesh.indices);
                render.register_mesh(&Self::chunk_key(raw_mesh.offset), gfx_mesh);
                self.render_chunks.insert(raw_mesh.offset);
                self.pending_render_chunks.remove(&raw_mesh.offset);
            });

        self.gfx_remove_queue.drain(..).for_each(|coord| {
            render.unregister_mesh(&Self::chunk_key(coord));
            self.render_chunks.remove(&coord);
            self.pending_render_chunks.remove(&coord);
        });
    }

    pub fn chunk_key(coord: glam::IVec3) -> String {
        format!("ch{}x{}x{}_mesh", coord.x, coord.y, coord.z)
    }

    fn chunk_in_range(&self, coord: glam::IVec3) -> bool {
        (coord.saturating_sub(self.center_chunk)).length_squared()
            < (self.view_distance * self.view_distance) as i32
    }

    fn chunk_surrounding(&mut self, center: glam::Vec3) -> glam::IVec3 {
        glam::ivec3(
            (center.x / self.chunk_width as f32).floor() as i32,
            0,
            (center.z / self.chunk_width as f32).floor() as i32,
        )
    }

    fn request_chunk_generation(&mut self, coord: glam::IVec3) {
        let Some(send) = &self.chunk_send
        else {
            log::error!("Chunk generation requested with no workers");
            return;
        };

        let request = ChunkRequest::Generate { coord };

        if let Err(err) = send.try_send(request) {
            log::debug!("Chunk generation sending error: {}", err);
            return;
        }

        self.pending_chunks.insert(coord);
    }

    fn request_chunk_meshing(&mut self, coord: glam::IVec3) {
        let Some(send) = &self.chunk_send
        else {
            log::error!("Chunk meshing requested with no workers");
            return;
        };

        let assistant = mesher::ChunkMeshingAssisant {
            chunk: sync::Arc::clone(&self.chunks[&coord]),
            neighbors: [
                self.chunks.get(&coord.with_z(&coord.z + 1)).cloned(),
                self.chunks.get(&coord.with_z(&coord.z - 1)).cloned(),
                self.chunks.get(&coord.with_x(&coord.x - 1)).cloned(),
                self.chunks.get(&coord.with_x(&coord.x + 1)).cloned(),
            ],
        };

        if !assistant.neighbors.iter().all(|neighbor| neighbor.is_some()) {
            return;
        }

        let request = ChunkRequest::Mesh {
            chunk: sync::Arc::clone(&self.chunks[&coord]),
            assistant,
        };

        if let Err(err) = send.try_send(request) {
            log::debug!("Chunk mesh sending error: {}", err);
            return;
        }

        self.pending_render_chunks.insert(coord);
    }

    fn async_chunk_response(
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
                    | ChunkRequest::Generate { coord } => {
                        let mut chunk = Chunk::new(coord, width, height);
                        terrain.form_chunk(&mut chunk);
                        let response = ChunkResponse::Generated { coord, chunk };
                        if let Err(err) = chunk_response.send(response) {
                            log::error!("Chunk terrain recieving error: {}", err);
                            return;
                        }
                        log::debug!("Chunk generated for {}", coord);
                    }
                    | ChunkRequest::Mesh { chunk, assistant } => {
                        let mesh = chunk.raw_mesh(&atlas, assistant);
                        let response = ChunkResponse::Meshed { coord: mesh.offset, raw_mesh: mesh };
                        if let Err(err) = chunk_response.send(response) {
                            log::error!("Chunk mesh recieving error: {}", err);
                            return;
                        }
                        log::debug!("Chunk mesh generated for {}", chunk.offset);
                    }
                    | ChunkRequest::Cleanup => {
                        return;
                    }
                }
            }
        });
    }
}
