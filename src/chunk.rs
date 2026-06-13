use crate::{block, engine::storage::buffer};

pub const CHUNK_WIDTH: usize = 32;
pub const CHUNK_HEIGHT: usize = 64;

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
}
