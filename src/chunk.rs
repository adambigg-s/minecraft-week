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
        let mut blocks = buffer::Buffer::new([CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_WIDTH]);
        blocks.fill(block::Block::Air);

        let (height, width) = (CHUNK_HEIGHT, CHUNK_WIDTH);

        Self { blocks, offset, height, width }
    }

    pub fn size(&self) -> glam::IVec3 {
        glam::ivec3(self.width as i32, self.height as i32, self.width as i32)
    }

    pub fn to_chunk_coords(&self, pos: glam::IVec3) -> glam::IVec3 {
        pos.rem_euclid(self.size())
    }

    pub fn chunk_coords(&self, pos: glam::IVec3) -> [usize; 3] {
        pos.to_array().map(|ele| ele as usize)
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
