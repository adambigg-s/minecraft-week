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

pub const CHUNK_WIDTH: usize = 32;
pub const CHUNK_HEIGHT: usize = 128;

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

    pub fn get(&self, coord: glam::IVec3) -> &block::Block {
        self.blocks.get(self.to_index(coord))
    }

    pub fn get_mut(&mut self, coord: glam::IVec3) -> &mut block::Block {
        self.blocks.get_mut(self.to_index(coord))
    }

    pub fn to_chunk_coords(&self, coord: glam::IVec3) -> glam::IVec3 {
        coord.rem_euclid(self.size())
    }

    pub fn raw_mesh(&self, atlas: &atlas::TextureAtlas) -> ChunkRawMesh {
        let mesher = mesher::ChunkMesher { chunk: self, atlas };
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

        ChunkRawMesh { vertices, indices }
    }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkRawMesh {
    vertices: Vec<mesher::TerrainVertex>,
    indices: Vec<u32>,
}

#[derive(Debug)]
pub enum ChunkRequest {
    Generate {
        coord: glam::IVec3,
        width: usize,
        height: usize,
        terrain: sync::Arc<terrain::TerrainGenerator>,
        atlas: sync::Arc<atlas::TextureAtlas>,
    },
    Cleanup,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkResponse {
    coord: glam::IVec3,
    chunk: Chunk,
    mesh: ChunkRawMesh,
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
    pub chunks: collections::HashMap<glam::IVec3, Chunk>,
    #[builder(default)]
    pub chunks_pending: collections::HashSet<glam::IVec3>,

    #[builder(default)]
    pub gfx_insert_queue: Vec<(glam::IVec3, ChunkRawMesh)>,
    #[builder(default)]
    pub gfx_remove_queue: Vec<glam::IVec3>,

    pub chunk_send: Option<mpsc::SyncSender<ChunkRequest>>,
    pub chunk_recv: Option<mpsc::Receiver<ChunkResponse>>,
}

impl ChunkManager {
    pub fn spawn_worker(&mut self) {
        let (send_tx, send_rx) = mpsc::sync_channel(self.view_distance * self.view_distance);
        let (recv_tx, recv_rx) = mpsc::channel();

        self.async_chunk_response(send_rx, recv_tx);

        self.chunk_send = Some(send_tx);
        self.chunk_recv = Some(recv_rx);
    }

    pub fn update_chunks(&mut self, center: glam::Vec3) {
        if let Some(recv) = &self.chunk_recv {
            while let Ok(ChunkResponse { coord, chunk, mesh }) = recv.try_recv() {
                self.chunks.insert(coord, chunk);
                self.chunks_pending.remove(&coord);
                self.gfx_insert_queue.push((coord, mesh));
            }
        }

        self.center_chunk = self.chunk_surrounding(center);
        let range = self.view_distance as i32;

        (-range..=range).for_each(|dz| {
            (-range..=range).for_each(|dx| {
                let coord = self.center_chunk + glam::ivec3(dx, 0, dz);
                if self.chunk_in_range(coord)
                    && !self.chunks_pending.contains(&coord)
                    && !self.chunks.contains_key(&coord)
                {
                    self.request_chunk(coord);
                }
            });
        });

        let removal = self
            .chunks
            .keys()
            .copied()
            .filter(|&coord| !self.chunk_in_range(coord))
            .collect::<Vec<glam::IVec3>>();
        removal.into_iter().for_each(|coord| {
            self.chunks.remove(&coord);
            self.chunks_pending.remove(&coord);
            self.gfx_remove_queue.push(coord);
        });
    }

    pub fn sync_gfx_chunks(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer) {
        self.gfx_insert_queue.drain(..).for_each(|(coord, raw_mesh)| {
            let gfx_mesh = mesh::GfxMesh::new(context, &raw_mesh.vertices, &raw_mesh.indices);
            render.register_mesh(&Self::chunk_key(coord), gfx_mesh);
        });
        self.gfx_remove_queue.drain(..).for_each(|coord| {
            render.unregister_mesh(&Self::chunk_key(coord));
        });
    }

    pub fn chunk_key(coord: glam::IVec3) -> String {
        format!("ch{}x{}x{}_mesh", coord.x, coord.y, coord.z)
    }

    fn chunk_in_range(&self, coord: glam::IVec3) -> bool {
        ((coord - self.center_chunk).length_squared() as usize) < self.view_distance * self.view_distance
    }

    fn chunk_surrounding(&mut self, center: glam::Vec3) -> glam::IVec3 {
        glam::ivec3(
            (center.x / self.chunk_width as f32).floor() as i32,
            0,
            (center.z / self.chunk_width as f32).floor() as i32,
        )
    }

    fn request_chunk(&mut self, coord: glam::IVec3) {
        let Some(send) = &self.chunk_send
        else {
            log::error!("Chunk requested with no workers");
            return;
        };

        let request = ChunkRequest::Generate {
            coord,
            width: self.chunk_width,
            height: self.chunk_height,
            terrain: sync::Arc::clone(&self.terrain),
            atlas: sync::Arc::clone(&self.atlas),
        };

        if let Err(err) = send.try_send(request) {
            log::debug!("Chunk sending error: {}", err);
            return;
        }

        self.chunks_pending.insert(coord);
    }

    fn async_chunk_response(
        &self,
        chunk_request: mpsc::Receiver<ChunkRequest>,
        chunk_response: mpsc::Sender<ChunkResponse>,
    ) {
        thread::spawn(move || {
            while let Ok(request) = chunk_request.recv() {
                match request {
                    | ChunkRequest::Generate { coord, width, height, terrain, atlas } => {
                        let mut chunk = Chunk::new(coord, width, height);
                        terrain.form_chunk(&mut chunk);
                        let mesh = chunk.raw_mesh(&atlas);

                        let response = ChunkResponse { coord, chunk, mesh };
                        if let Err(err) = chunk_response.send(response) {
                            log::error!("Chunk recieving error: {}", err);
                            return;
                        }
                    }
                    | ChunkRequest::Cleanup => {
                        return;
                    }
                }
            }
        });
    }
}
