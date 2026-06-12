use noise::NoiseFn;

use crate::{block, chunk};

#[derive(bon::Builder, Debug)]
pub struct TerrainGenerator {
    pub noise: noise::OpenSimplex,
    pub scale: f64,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = noise::OpenSimplex::new(seed);
        let scale = 1e-2;

        Self { noise, scale }
    }

    pub fn sample(&self, coords: glam::DVec3) -> f32 {
        ((self.noise.get([coords.x, coords.z]) + 1.0) * 0.5) as f32
    }

    pub fn new_chunk(&self, offset: glam::IVec3) -> chunk::Chunk {
        let mut chunk = chunk::Chunk::new(offset);
        let base = offset * chunk.size();

        for z in 0..chunk.width as i32 {
            for x in 0..chunk.width as i32 {
                let real = base + glam::ivec3(x, 0, z);
                let v3 = real.as_dvec3() * self.scale;
                let mut height = (self.sample(v3) * chunk.height as f32) as i32;
                height = height.clamp(1, chunk.height as i32);
                for y in 0..height {
                    *chunk.blocks.get_mut([x as usize, y as usize, z as usize]) = block::Block::random();
                }
            }
        }

        chunk
    }
}
