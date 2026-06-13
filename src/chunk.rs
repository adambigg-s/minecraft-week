use std::collections;

use crate::{
    atlas, block,
    engine::storage::buffer,
    mesher,
    render::{self, mesh},
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
    pub chunks: collections::HashMap<glam::IVec3, Chunk>,
    pub view_distance: u32,
    pub player_chunk: glam::IVec3,
}

impl ChunkManager {}
