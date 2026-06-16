use noise::NoiseFn;

use crate::{block, chunk};

pub enum Biome {
    Ocean,
    Beach,
    Meadow,
    Forest,
    Mountain,
    Badlands,
}

pub struct GeologyRules {}

#[derive(bon::Builder, Debug)]
pub struct TerrainGenerator {
    pub noise: noise::Perlin,
}

pub fn integer_weighted_sum<const N: usize>(values: [f64; N], weights: [i32; N]) -> f64 {
    let total_weight = weights.iter().sum::<i32>() as f64;
    values
        .iter()
        .zip(weights.iter())
        .map(|(&val, &weight)| val * (weight as f64) / total_weight)
        .sum()
}

// terrain shaping
// water filling
// surface replacement
// decorators
impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = noise::Perlin::new(seed);

        Self { noise }
    }

    pub fn form_chunk(&self, chunk: &mut chunk::Chunk) {
        use block::Block::*;

        let height = chunk.height as f64;
        let sea_level = (height * 0.5) as i32;
        let chunk_offset = chunk.offset * chunk.size();
        for z in 0..chunk.width as i32 {
            for x in 0..chunk.width as i32 {
                let coord = (glam::ivec3(x, 0, z) + chunk_offset).as_dvec3();

                let continent = self.sample_fbm_2d([coord.x, coord.z], 4, 0.002);
                let detail = self.sample_fbm_2d([coord.x + 1000.0, coord.z + 1000.0], 6, 0.01);
                let mountain = self.sample_fbm_2d([coord.x + 10000.0, coord.z + 10000.0], 8, 0.15).powf(0.3);

                let height = integer_weighted_sum([continent, detail, mountain], [1, 0, 0]);

                for y in 0..height as i32 {
                    *chunk.get_mut(glam::ivec3(x, y, z)) = Stone;
                }
            }
        }
    }

    fn sample_fbm_2d(&self, point: [f64; 2], octaves: usize, freq: f64) -> f64 {
        let mut total = 0.0;
        let mut max = 0.0;
        let mut amp = 1.0;
        let mut freq = freq;

        (0..octaves).for_each(|_| {
            total += self.noise.get(point.map(|val| val * freq)) * amp;
            max += amp;
            amp *= 0.5;
            freq *= 2.0;
        });

        ((total / max) + 1.0) * 0.5
    }

    pub fn sample_fbm_3d(&self, point: [f64; 3], octaves: usize, freq: f64) -> f64 {
        let mut total = 0.0;
        let mut max = 0.0;
        let mut amp = 1.0;
        let mut freq = freq;

        (0..octaves).for_each(|_| {
            total += self.noise.get(point.map(|val| val * freq)) * amp;
            max += amp;
            amp *= 0.5;
            freq *= 2.0;
        });

        ((total / max) + 1.0) * 0.5
    }
}
