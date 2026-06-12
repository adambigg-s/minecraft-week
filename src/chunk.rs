use crate::{block, engine::storage::buffer};

pub const CHUNK_S: usize = 32;

#[derive(bon::Builder, Debug)]
pub struct Chunk {
    pub blocks: buffer::Buffer<block::Block, 3>,
    pub height: usize,
    pub width: usize,
}

impl Chunk {
    pub fn new() -> Self {
        let mut blocks = buffer::Buffer::new([CHUNK_S, CHUNK_S, CHUNK_S]);
        blocks.fill(block::Block::Air);

        let (height, width) = (CHUNK_S, CHUNK_S);

        Self { blocks, height, width }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_random_chunk() -> Chunk {
    let mut chunk = Chunk::new();
    for i in 0..32 {
        for j in 0..32 {
            let fx = j as f32 / chunk.width as f32;
            let fy = i as f32 / chunk.width as f32;

            let mut height = (chunk.height as f32 * (fx * 4.0).sin() * (fy * 3.0).cos()) as i32;
            height = height.clamp(1, chunk.height as i32 - 1);

            for k in 0..height {
                let block = block::Block::from(rand::random::<u8>() % block::Block::BlockCounter as u8);
                *chunk.blocks.get_mut([i, k as usize, j]) = block;
            }
        }
    }

    chunk
}
