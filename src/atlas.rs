use std::{collections, fs};

use image::GenericImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlockFace {
    Top,
    Side,
    Bottom,
}

pub struct TextureAtlas {
    pub tile_size: u32,
    pub image: image::RgbaImage,
    pub offsets: collections::HashMap<(String, BlockFace), glam::Vec2>,
}

impl TextureAtlas {
    pub fn new(path: &str, tile_sizes: usize) -> anyhow::Result<Self> {
        let mut block_images: collections::HashMap<
            String,
            collections::HashMap<BlockFace, image::DynamicImage>,
        > = collections::HashMap::new();
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            let block_name = file_name.split("_").nth(0).unwrap().to_string();
            let image = image::open(entry.path())?;

            let face = if file_name.contains("_top") {
                BlockFace::Top
            }
            else if file_name.contains("_bot") {
                BlockFace::Bottom
            }
            else {
                BlockFace::Side
            };

            block_images.entry(block_name).or_default().insert(face, image);
        }

        let total_faces = block_images.values().map(|face| face.len()).sum::<usize>();
        let side_tiles = (total_faces as f32).sqrt().ceil() as u32;
        let atlas_dim = side_tiles * tile_sizes as u32;

        let mut atlas_image = image::RgbaImage::new(atlas_dim, atlas_dim);
        let mut offsets = collections::HashMap::new();

        let mut curr_idx = 0;
        for (block_name, face) in block_images {
            for (face, img) in face {
                let x = (curr_idx % side_tiles) * tile_sizes as u32;
                let y = (curr_idx / side_tiles) * tile_sizes as u32;
                atlas_image.copy_from(&img, x, y)?;

                let uv = glam::vec2(x as f32 / atlas_dim as f32, y as f32 / atlas_dim as f32);
                offsets.insert((block_name.clone(), face), uv);

                curr_idx += 1;
            }
        }

        

        Ok(Self {
            tile_size: tile_sizes as u32,
            image: atlas_image,
            offsets,
        })
    }
}
