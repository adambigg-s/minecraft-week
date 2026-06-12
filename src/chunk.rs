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
}

pub fn generate_random_chunk(offset: glam::IVec3) -> Chunk {
    let mut chunk = Chunk::new(offset);
    for i in 0..32 {
        for j in 0..32 {
            let fx = j as f32 / chunk.width as f32;
            let fy = i as f32 / chunk.width as f32;

            let mut height = (chunk.height as f32 * (fx * 4.0).sin().abs() * (fy * 3.0).cos().abs()) as i32;
            height = height.clamp(1, chunk.height as i32 - 1);

            for k in 0..height {
                *chunk.blocks.get_mut([i, k as usize, j]) = block::Block::random();
            }
        }
    }

    chunk
}
