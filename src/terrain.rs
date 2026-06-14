use noise::NoiseFn;
use rand::{SeedableRng, rngs};

use crate::{block, chunk, engine::storage::buffer};

#[derive(bon::Builder, Debug, Clone)]
pub struct TerrainGenerator {
    pub heightmap_noise: noise::Perlin,
    pub warping_noise: noise::Perlin,
    pub rand: rngs::SmallRng,
    pub freq: f64,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let heightmap_noise = noise::Perlin::new(seed);
        let warping_noise = noise::Perlin::new(seed + 67);
        let freq = 0.005;
        let rand = rngs::SmallRng::seed_from_u64(seed as u64);

        Self { heightmap_noise, warping_noise, rand, freq }
    }

    pub fn form_chunk(&self, chunk: &mut chunk::Chunk) {
        use block::Block::*;

        let chunk_height = chunk.height as f64;
        let heightmap = self.heightmap(chunk.width, chunk.offset, self.freq, 3);
        let roughmap = self.heightmap(chunk.width, chunk.offset, self.freq * 4.0, 5);
        for z in 0..heightmap.size()[1] {
            for x in 0..heightmap.size()[0] {
                let height = *heightmap.get([x, z]);
                let rough = *roughmap.get([x, z]);
                let extra_rought = rough.powf(0.3);

                let terrain_height = height * chunk_height;
                let rough_height = rough * chunk_height;
                let extra_rought = extra_rought * chunk_height;

                let height = terrain_height + rough_height / 12.0 + extra_rought / 4.0;

                let height = (height as i32).clamp(1, chunk.height as i32 - 1);
                let dirt_thickness = (rough * 1.3) as i32;

                let water = (chunk_height * 0.45) as i32;

                for y in 0..chunk.height as i32 {
                    let pos = glam::ivec3(x as i32, y, z as i32);
                    let block = if y > height {
                        if y <= water { Water } else { Air }
                    }
                    else if y == height {
                        if y <= water + 2 { Sand } else { Grass }
                    }
                    else if y >= height - dirt_thickness {
                        if height <= water + 2 { Sand } else { Dirt }
                    }
                    else {
                        Stone
                    };

                    *chunk.get_mut(pos) = block;
                }
            }
        }
    }

    fn heightmap(
        &self,
        size: usize,
        origin: glam::IVec3,
        freq: f64,
        octaves: usize,
    ) -> buffer::Buffer<f64, 2> {
        let mut out = buffer::Buffer::new([size, size]);
        let base = origin * glam::ivec3(size as i32, 0, size as i32);
        for x in 0..size {
            for z in 0..size {
                let point = (base + glam::ivec3(x as i32, 0, z as i32)).as_dvec3();

                let height = self.sample_fbm(point, octaves, freq);
                *out.get_mut([x, z]) = height;
            }
        }
        out
    }

    fn sample_fbm(&self, point: glam::DVec3, octaves: usize, freq: f64) -> f64 {
        let mut total = 0.0;
        let mut max = 0.0;
        let mut amp = 1.0;
        let mut freq = freq;

        (0..octaves).for_each(|_| {
            total += self.heightmap_noise.get([point.x * freq, point.z * freq]) * amp;
            max += amp;
            amp *= 0.5;
            freq *= 2.0;
        });

        ((total / max) + 1.0) * 0.5
    }
}
