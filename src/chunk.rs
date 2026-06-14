use std::{
    collections,
    sync::{self, mpsc},
    thread, time,
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

    pub fn get(&self, coord: glam::IVec3) -> &block::Block {
        self.blocks.get(self.idx(coord))
    }

    pub fn get_mut(&mut self, coord: glam::IVec3) -> &mut block::Block {
        self.blocks.get_mut(self.idx(coord))
    }

    pub fn idx(&self, coord: glam::IVec3) -> [usize; 3] {
        coord.to_array().map(|ele| ele as usize)
    }

    pub fn to_chunk_coords(&self, coord: glam::IVec3) -> glam::IVec3 {
        coord.rem_euclid(self.size())
    }

    pub fn mesh(&self, context: &render::GfxContext, atlas: &atlas::TextureAtlas) -> mesh::GfxMesh {
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
        let indices = &rectilinear.indices;

        mesh::GfxMesh::new(context, &vertices, indices)
    }
}

#[derive(Debug)]
pub enum ChunkRequest {
    Create { coord: glam::IVec3 },
    Cleanup,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkResponse {
    pub coord: glam::IVec3,
    pub chunk: Chunk,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkManager {
    #[builder(default)]
    pub chunks: collections::HashMap<glam::IVec3, Chunk>,

    pub chunk_width: usize,
    pub chunk_height: usize,

    #[builder(default)]
    pub gfx_mesh_queue: Vec<glam::IVec3>,

    #[builder(default)]
    pub gfx_remove_queue: Vec<glam::IVec3>,

    pub atlas: sync::Arc<atlas::TextureAtlas>,
    pub terrain: sync::Arc<terrain::TerrainGenerator>,
    pub view_distance: u32,

    #[builder(default = glam::IVec3::MAX)]
    pub center_chunk: glam::IVec3,

    pub chunk_send: Option<mpsc::SyncSender<ChunkRequest>>,
    pub chunk_recv: Option<mpsc::Receiver<ChunkResponse>>,
}

impl ChunkManager {
    // pub fn spawn_worker(&mut self, terrain: terrain::TerrainGenerator) {
    //     let (req_tx, req_rx) = mpsc::sync_channel::<ChunkRequest>(64);
    //     let (res_tx, res_rx) = mpsc::channel::<ChunkResponse>();

    //     thread::spawn(move || {
    //         while let Ok(message) = req_rx.recv() {
    //             match message {
    //                 | ChunkRequest::Create { coord } => {
    //                     let mut chunk = Chunk::new(coord, self.chunk_width, self.chunk_height);
    //                     terrain.form_chunk(&mut chunk);
    //                     if res_tx.send(ChunkResponse { coord, chunk }).is_err() {
    //                         break;
    //                     }
    //                 }
    //                 | ChunkRequest::Cleanup => {
    //                     break;
    //                 }
    //             }
    //         }
    //     });

    //     self.chunk_send = Some(req_tx);
    //     self.chunk_recv = Some(res_rx);
    // }

    fn request_chunk(&mut self, coord: glam::IVec3) {
        if self.chunks.contains_key(&coord) {
            return;
        }

        self.chunks.insert(coord, Chunk::new(coord, self.chunk_width, self.chunk_height));

        if let Some(tx) = &self.chunk_send
            && let Err(err) = tx.try_send(ChunkRequest::Create { coord })
        {
            log::error!("Chunk request queue full, dropping {:?}: {}", coord, err);
        }
    }

    fn drain_finished_chunks(&mut self) {
        let Some(rx) = &self.chunk_recv
        else {
            return;
        };

        while let Ok(ChunkResponse { coord, chunk }) = rx.try_recv() {
            self.chunks.insert(coord, chunk);
            self.gfx_mesh_queue.push(coord);
        }
    }

    pub fn update_chunks(&mut self, center: glam::Vec3) {
        let center_chunk = glam::ivec3(
            (center.x / self.chunk_width as f32) as i32,
            0,
            (center.z / self.chunk_width as f32) as i32,
        );
        if center_chunk == self.center_chunk {
            return;
        }
        self.center_chunk = center_chunk;

        let range = self.view_distance as i32;

        let mut removal = Vec::new();
        self.chunks.keys().for_each(|&coord| {
            let dx = coord.x - center_chunk.x;
            let dz = coord.z - center_chunk.z;
            if (dx * dx + dz * dz) >= range * range {
                self.gfx_remove_queue.push(coord);
                removal.push(coord);
            }
        });
        removal.iter().for_each(|coord| {
            self.chunks.remove(coord);
        });

        for dx in -range..range {
            for dz in -range..range {
                if (dx * dx + dz * dz) >= range * range {
                    continue;
                }

                let coord = center_chunk + glam::ivec3(dx, 0, dz);
                // self.request_chunk(coord);
                self.generate_chunk(coord);
            }
        }
    }

    pub fn sync_gfx_chunks(&mut self, context: &render::GfxContext, render: &mut render::GfxRenderer) {
        self.gfx_remove_queue.iter().for_each(|&coord| {
            let key = self.chunk_key(coord);
            render.unregister_mesh(&key);
        });
        self.gfx_remove_queue.clear();

        self.gfx_mesh_queue.iter().for_each(|&coord| {
            let start_time = time::Instant::now();
            let chunk = &self.chunks[&coord];
            let key = self.chunk_key(coord);
            render.register_mesh(&key, chunk.mesh(context, &self.atlas));
            log::debug!("Chunk meshing time: {}", start_time.elapsed().as_millis());
        });
        self.gfx_mesh_queue.clear();
    }

    pub fn chunk_key(&self, coord: glam::IVec3) -> String {
        format!("ch{}x{}x{}_mesh", coord.x, coord.y, coord.z)
    }

    fn generate_chunk(&mut self, coord: glam::IVec3) {
        if self.chunks.contains_key(&coord) {
            return;
        }

        let start_time = time::Instant::now();
        let mut chunk = Chunk::new(coord, self.chunk_width, self.chunk_height);
        self.terrain.form_chunk(&mut chunk);
        self.chunks.insert(coord, chunk);
        self.gfx_mesh_queue.push(coord);
        log::debug!("Chunk generation time: {}", start_time.elapsed().as_millis());
    }

    // pub fn update_chunks(&mut self, center: glam::Vec3) {
    //     let center_chunk = glam::ivec3(
    //         (center.x / self.chunk_width as f32) as i32,
    //         0,
    //         (center.z / self.chunk_width as f32) as i32,
    //     );
    //     if center_chunk == self.center_chunk {
    //         return;
    //     }
    //     self.center_chunk = center_chunk;

    //     let range = self.view_distance as i32;

    //     let mut removal = Vec::new();
    //     self.chunks.keys().for_each(|&coord| {
    //         let dx = coord.x - center_chunk.x;
    //         let dz = coord.z - center_chunk.z;
    //         if (dx * dx + dz * dz) >= range * range {
    //             self.gfx_remove_queue.push(coord);
    //             removal.push(coord);
    //         }
    //     });
    //     removal.iter().for_each(|coord| {
    //         self.chunks.remove(coord);
    //     });

    //     for dx in -range..range {
    //         for dz in -range..range {
    //             if (dx * dx + dz * dz) >= range * range {
    //                 continue;
    //             }

    //             let coord = center_chunk + glam::ivec3(dx, 0, dz);
    //             self.generate_chunk(coord);
    //         }
    //     }
    // }

    // pub fn sync_gfx_chunks(&mut self, context: &render::GfxContext, render: &mut render::GfxRenderer) {
    //     self.gfx_remove_queue.iter().for_each(|&coord| {
    //         let key = self.chunk_key(coord);
    //         render.unregister_mesh(&key);
    //     });
    //     self.gfx_remove_queue.clear();

    //     self.gfx_mesh_queue.iter().for_each(|&coord| {
    //         let start_time = time::Instant::now();
    //         let chunk = &self.chunks[&coord];
    //         let key = self.chunk_key(coord);
    //         render.register_mesh(&key, chunk.mesh(context, &self.atlas));
    //         log::debug!("Chunk meshing time: {}", start_time.elapsed().as_millis());
    //     });
    //     self.gfx_mesh_queue.clear();
    // }

    // pub fn chunk_key(&self, coord: glam::IVec3) -> String {
    //     format!("ch{}x{}x{}_mesh", coord.x, coord.y, coord.z)
    // }

    // fn generate_chunk(&mut self, coord: glam::IVec3) {
    //     if self.chunks.contains_key(&coord) {
    //         return;
    //     }

    //     let start_time = time::Instant::now();
    //     let mut chunk = Chunk::new(coord);
    //     self.terrain.form_chunk(&mut chunk);
    //     self.chunks.insert(coord, chunk);
    //     self.gfx_mesh_queue.push(coord);
    //     log::debug!("Chunk generation time: {}", start_time.elapsed().as_millis());
    // }
}
