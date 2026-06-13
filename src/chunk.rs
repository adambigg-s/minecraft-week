use std::{collections, time};

use crate::{
    atlas, block,
    engine::storage::buffer,
    mesher,
    render::{self, mesh},
    terrain,
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
    pub fn new(offset: glam::IVec3) -> Self {
        let blocks = buffer::Buffer::new_zeroed([CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_WIDTH]);
        let (height, width) = (CHUNK_HEIGHT, CHUNK_WIDTH);

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

    pub atlas: atlas::TextureAtlas,
    pub terrain: terrain::TerrainGenerator,
    pub view_distance: u32,

    #[builder(default = glam::IVec3::MAX)]
    pub center_chunk: glam::IVec3,
}

impl ChunkManager {
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
        format!("ch{}x{}_mesh", coord.x, coord.z)
    }

    fn generate_chunk(&mut self, coord: glam::IVec3) {
        if self.chunks.contains_key(&coord) {
            return;
        }

        let start_time = time::Instant::now();
        let chunk = self.terrain.new_chunk(coord);
        self.chunks.insert(coord, chunk);
        self.gfx_mesh_queue.push(coord);
        log::debug!("Chunk generation time: {}", start_time.elapsed().as_millis());
    }
}
