use std::{collections, fs};

use image::GenericImage;

use crate::{engine::storage::buffer, mesher};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlockTextureFace {
    Top,
    #[default]
    Side,
    Bottom,
}

#[derive(bon::Builder, Debug)]
pub struct TextureAtlas {
    pub atlas_size: u32,
    pub tile_size: u32,
    pub atlas: image::RgbaImage,
    pub offsets: collections::HashMap<String, collections::HashMap<BlockTextureFace, glam::Vec2>>,
}

impl TextureAtlas {
    pub fn new(directory: &str, tile_size: u32) -> anyhow::Result<Self> {
        let images = Self::collect_textures(directory, tile_size)?;

        let image_count = images.values().map(|faces| faces.len() as u32).sum::<u32>();
        let tiles_per_side = image_count.isqrt() + 1;
        let atlas_size = (tiles_per_side * tile_size).next_power_of_two();

        let mut atlas = image::RgbaImage::new(atlas_size, atlas_size);
        let mut offsets: collections::HashMap<String, collections::HashMap<BlockTextureFace, glam::Vec2>> =
            collections::HashMap::new();

        let mut images = images.iter().collect::<Vec<_>>();
        images.sort_unstable_by_key(|(a, _)| *a);

        let index_assistant = buffer::Buffer::<(), 2>::new([tiles_per_side as usize; 2]);
        let mut current_tile = 0;
        for (block_name, faces) in images {
            let mut faces = faces.iter().collect::<Vec<_>>();
            faces.sort_unstable_by_key(|(a, _)| *a);

            for (&face, image) in faces {
                let [x, y] = index_assistant.delinearize(current_tile).map(|val| val as u32 * tile_size);

                atlas.copy_from(image, x, y)?;
                log::info!("Block written into texture atlas: ({}, {:?})", block_name, face,);

                let uv = glam::vec2(x as f32, y as f32) / atlas_size as f32;
                offsets.entry(block_name.to_owned()).or_default().insert(face, uv);

                current_tile += 1;
            }
        }
        log::warn!("Texture atlas created: {} images at {} pixels", image_count, atlas_size);

        Ok(Self { atlas_size, tile_size, atlas, offsets })
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        self.atlas.save(path)?;
        Ok(())
    }

    pub fn conform_uvs(&self, uvs: &mut [glam::Vec2], name: &str, face: mesher::Face) {
        let face_texture = match face {
            | mesher::Face::Top => BlockTextureFace::Top,
            | mesher::Face::Bottom => BlockTextureFace::Bottom,
            | _ => BlockTextureFace::Side,
        };

        let scale = self.tile_size as f32 / self.atlas_size as f32;
        let offset = match self.offsets.get(name) {
            | Some(blocks) => match blocks.get(&face_texture) {
                | Some(offset) => *offset,
                | None => match blocks.get(&BlockTextureFace::Side) {
                    | Some(offset) => *offset,
                    | None => {
                        log::error!("No default side texture for block: {}", name);
                        glam::Vec2::ZERO
                    }
                },
            },
            | None => {
                log::error!("No texture images found for block: {}", name);
                glam::Vec2::ZERO
            }
        };

        for uv in uvs {
            *uv *= scale;
            *uv += offset;
        }
    }

    fn collect_textures(
        directory: &str,
        tile_size: u32,
    ) -> anyhow::Result<collections::HashMap<String, collections::HashMap<BlockTextureFace, image::RgbaImage>>>
    {
        let mut images: collections::HashMap<
            String,
            collections::HashMap<BlockTextureFace, image::RgbaImage>,
        > = collections::HashMap::new();

        let entries = fs::read_dir(directory)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|extension| extension.to_str()) != Some("png") {
                log::error!("Attempted read on invalid file: {:?}", path);
                continue;
            }

            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid file name: {:?}", path))?;

            let (block_name, tile_type) = stem
                .split_once('_')
                .ok_or_else(|| anyhow::anyhow!("File name has no '_' separator: {:?}", path))?;

            let face = match tile_type {
                | "top" | "t" => BlockTextureFace::Top,
                | "side" | "s" => BlockTextureFace::Side,
                | "bottom" | "bot" | "b" => BlockTextureFace::Bottom,
                | _ => BlockTextureFace::Side,
            };

            let image = image::open(&path)?.to_rgba8();

            if image.width() != tile_size || image.height() != tile_size {
                log::error!("Invalid image size: {:?} at {}x{}", path, image.width(), image.height());
                continue;
            }

            images.entry(block_name.to_string()).or_default().insert(face, image);
        }

        Ok(images)
    }
}
