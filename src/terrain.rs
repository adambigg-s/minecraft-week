use noise::NoiseFn;

use crate::{block, chunk};

#[derive(Debug)]
pub enum Biome {
    Ocean,
    Beach,
    Meadow,
    Forest,
    Mountain,
    Badlands,
}

#[derive(bon::Builder, Debug)]
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

    pub fn generate_tree(&self, chunk: &mut chunk::Chunk, x: i32, y: i32, z: i32) {
        use block::Block::*;

        for dy in 0..4 {
            let log_pos = glam::ivec3(x, y + dy, z);
            if chunk.get(log_pos) == &Air {
                *chunk.get_mut(log_pos) = Log;
            }
        }

        for ly in 2..4 {
            for lx in -2..=2_i32 {
                for lz in -2..=2_i32 {
                    if lx.abs() == 2 && lz.abs() == 2 {
                        continue;
                    }

                    let leaf_pos = glam::ivec3(x + lx, y + ly, z + lz);
                    if chunk.check_index(leaf_pos) && chunk.get(leaf_pos) == &Air {
                        *chunk.get_mut(leaf_pos) = Leaf;
                    }
                }
            }
        }

        for lx in -1..=1_i32 {
            for lz in -1..=1_i32 {
                if lx.abs() == 1 && lz.abs() == 1 {
                    continue;
                }

                let leaf_pos = glam::ivec3(x + lx, y + 4, lz + z);
                if chunk.check_index(leaf_pos) && chunk.get(leaf_pos) == &Air {
                    *chunk.get_mut(leaf_pos) = Leaf;
                }
            }
        }
    }

    pub fn form_chunk(&self, chunk: &mut chunk::Chunk) {
        use block::Block::*;

        let height = chunk.height as f64;
        let sea_level = (height * 0.45) as i32;
        let chunk_offset = chunk.offset * chunk.size();
        for z in 0..chunk.width as i32 {
            for x in 0..chunk.width as i32 {
                let gcoord = (glam::ivec3(x, 0, z) + chunk_offset).as_dvec3();

                let continent = self.sample_fbm_2d([gcoord.x, gcoord.z], 4, 0.002);
                let continent_height = (continent * height) as i32;
                let detail = self.sample_fbm_2d([gcoord.x + 1000.0, gcoord.z + 1000.0], 6, 0.01);
                let mountain =
                    self.sample_fbm_2d([gcoord.x + 10000.0, gcoord.z + 10000.0], 8, 0.04).powf(5.0);

                let mut terrain_height = integer_weighted_sum([continent, detail, mountain], [3, 4, 1]);

                let decorator = self.sample_2d([gcoord.x + 99000.0, gcoord.z + 99000.0], 1.1);
                let decorator2 = self.sample_2d([gcoord.x + 89000.0, gcoord.z + 89000.0], 1.1);
                let decorator3 = self.sample_2d([gcoord.x + -300.0, gcoord.z + -300.0], 0.3);

                let cliff_subtrator = self.sample_fbm_2d([gcoord.x + 5000.0, gcoord.z + 5000.0], 1, 0.03);
                if cliff_subtrator > 0.75 {
                    terrain_height -= detail / 8.0;
                    terrain_height += mountain / 4.0;
                }

                let height = (terrain_height * height).min(height - 1.0);
                for y in 0..height as i32 {
                    let coord = glam::ivec3(x, y, z);
                    let gcoord = (coord + chunk_offset).as_dvec3();

                    *chunk.get_mut(glam::ivec3(x, y, z)) = Stone;

                    if self.sample_fbm_3d([gcoord.x, gcoord.y, gcoord.z], 2, 0.01) < 0.45
                        && y > continent_height
                    {
                        *chunk.get_mut(glam::ivec3(x, y, z)) = Air;
                    }
                }

                for y in 0..sea_level {
                    let coord = glam::ivec3(x, y, z);
                    if chunk.get(coord) == &Air {
                        *chunk.get_mut(coord) = Water;
                    }
                }

                for y in ((height - 3.0) as i32).max(0)..=height as i32 {
                    let coord = glam::ivec3(x, y, z);
                    if y == height as i32 {
                        *chunk.get_mut(coord) = Grass;
                    }
                    else {
                        *chunk.get_mut(coord) = Dirt;
                    }
                }

                for y in sea_level - 2..sea_level + 2 {
                    let coord = glam::ivec3(x, y, z);
                    if chunk.get(coord) != &Water && chunk.get(coord) != &Air {
                        *chunk.get_mut(coord) = Sand
                    }
                }

                if decorator > 0.98 {
                    let coord = glam::ivec3(x, height as i32, z);
                    if chunk.get(coord) == &Grass && chunk.get(coord.with_y(height as i32 + 1)) == &Air {
                        self.generate_tree(chunk, x, height as i32 + 1, z);
                    }
                }

                if decorator2 > 0.95 && decorator > 0.5 {
                    let coord = glam::ivec3(x, height as i32 + 1, z);
                    if chunk.get(coord) == &Air {
                        *chunk.get_mut(coord) = RedFlower
                    }
                }
                if decorator2 > 0.95 && decorator < 0.5 {
                    let coord = glam::ivec3(x, height as i32 + 1, z);
                    if chunk.get(coord) == &Air {
                        *chunk.get_mut(coord) = BlueFlower
                    }
                }

                if decorator3 > 0.85 {
                    let coord = glam::ivec3(x, height as i32 + 1, z);
                    if chunk.get(coord) == &Air {
                        *chunk.get_mut(coord) = Shrub
                    }
                }

                for y in sea_level..height as i32 {
                    let ore = self.sample_3d([gcoord.x, y as f64, gcoord.z], 0.1);
                    if ore > 0.75 {
                        let coord = glam::ivec3(x, y, z);
                        if chunk.get(coord) == &Stone {
                            *chunk.get_mut(coord) = Gravel
                        }
                    }
                }
            }
        }
    }

    fn sample_2d(&self, point: [f64; 2], freq: f64) -> f64 {
        (self.noise.get(point.map(|val| val * freq)) + 1.0) * 0.5
    }

    fn sample_3d(&self, point: [f64; 3], freq: f64) -> f64 {
        (self.noise.get(point.map(|val| val * freq)) + 1.0) * 0.5
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
