use glam::Vec3Swizzles;
use noise::NoiseFn;

use crate::{block, chunk};

#[derive(bon::Builder, Debug)]
pub struct TerrainGenerator {
    pub noise: noise::OpenSimplex,
    pub scale: f64,
    pub water: i32,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = noise::OpenSimplex::new(seed);
        let scale = 0.005;
        let water = 64;

        Self { noise, scale, water }
    }

    // https://thebookofshaders.com/13/
    pub fn sample_fbm(&self, coords: glam::DVec2) -> f64 {
        let mut freq = self.scale;
        let mut amp = 1.0;
        let mut max = 0.0;
        let mut total = 0.0;

        for _ in 0..8 {
            total += self.noise.get([coords.x * freq, coords.y * freq]) * amp;
            max += amp;
            amp *= 0.5;
            freq *= 2.0;
        }

        ((total / max) + 1.0) * 0.5
    }

    pub fn sample(&self, coords: glam::DVec3) -> f32 {
        ((self.noise.get([coords.x, coords.z]) + 1.0) * 0.5) as f32
    }

    pub fn new_chunk(&self, location: glam::IVec3) -> chunk::Chunk {
        use block::Block::*;

        let mut chunk = chunk::Chunk::new(location);
        let base = location * chunk.size();

        for z in 0..chunk.width as i32 {
            for x in 0..chunk.width as i32 {
                let real = base + glam::ivec3(x, 0, z);
                let noise = self.sample_fbm(real.as_dvec3().xz());
                let height = noise.powf(1.5);

                let min_height = 32.0;
                let max_height = 128.0;
                let mut height = (min_height + height * (max_height - min_height)) as i32;
                height = height.clamp(1, chunk.height as i32 - 1);

                for y in 0..chunk.height as i32 {
                    let mut block_type = if y > height {
                        if y <= self.water { Water } else { Air }
                    }
                    else if y == height {
                        if y > self.water + 1 {
                            Grass
                        }
                        else if y >= self.water - 1 {
                            Sand
                        }
                        else {
                            Dirt
                        }
                    }
                    else if y > height - 4 {
                        if height <= self.water + 1 { Sand } else { Dirt }
                    }
                    else {
                        Stone
                    };

                    if rand::random_bool(0.01) && block_type != Air {
                        block_type = block::Block::random();
                    }

                    if block_type != Air {
                        *chunk.blocks.get_mut([x as usize, y as usize, z as usize]) = block_type;
                    }
                }
            }
        }

        chunk
    }
}
