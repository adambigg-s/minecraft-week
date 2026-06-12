use std::mem;

use wgpu::vertex_attr_array;

use crate::{
    atlas, block, chunk,
    render::{self, mesh},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Back,
    Front,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Top,
        Face::Bottom,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];

    #[rustfmt::skip]
    pub fn neighbor_offset(&self) -> glam::IVec3 {
        match self {
            | Face::Top    => glam::ivec3(0, 1, 0),
            | Face::Bottom => glam::ivec3(0, -1, 0),
            | Face::Left   => glam::ivec3(-1, 0, 0),
            | Face::Right  => glam::ivec3(1, 0, 0),
            | Face::Back   => glam::ivec3(0, 0, 1),
            | Face::Front  => glam::ivec3(0, 0, -1),
        }
    }

    #[rustfmt::skip]
    pub fn normal(&self) -> glam::Vec3 {
        match self {
            | Face::Top    => glam::Vec3::Y,
            | Face::Bottom => glam::Vec3::NEG_Y,
            | Face::Left   => glam::Vec3::X,
            | Face::Right  => glam::Vec3::NEG_X,
            | Face::Back   => glam::Vec3::Z,
            | Face::Front  => glam::Vec3::NEG_Z,
        }
    }

    pub fn corners(&self) -> [(glam::IVec3, glam::IVec2); 4] {
        use glam::{ivec2, ivec3};

        match self {
            | Face::Top => [
                (ivec3(0, 1, 1), ivec2(0, 0)),
                (ivec3(0, 1, 0), ivec2(0, 1)),
                (ivec3(1, 1, 1), ivec2(1, 0)),
                (ivec3(1, 1, 0), ivec2(1, 1)),
            ],
            | Face::Bottom => [
                (ivec3(0, 0, 0), ivec2(0, 0)),
                (ivec3(0, 0, 1), ivec2(0, 1)),
                (ivec3(1, 0, 0), ivec2(1, 0)),
                (ivec3(1, 0, 1), ivec2(1, 1)),
            ],
            | Face::Left => [
                (ivec3(0, 1, 1), ivec2(0, 0)),
                (ivec3(0, 0, 1), ivec2(0, 1)),
                (ivec3(0, 1, 0), ivec2(1, 0)),
                (ivec3(0, 0, 0), ivec2(1, 1)),
            ],
            | Face::Right => [
                (ivec3(1, 1, 0), ivec2(0, 0)),
                (ivec3(1, 0, 0), ivec2(0, 1)),
                (ivec3(1, 1, 1), ivec2(1, 0)),
                (ivec3(1, 0, 1), ivec2(1, 1)),
            ],
            | Face::Back => [
                (ivec3(1, 1, 1), ivec2(0, 0)),
                (ivec3(1, 0, 1), ivec2(0, 1)),
                (ivec3(0, 1, 1), ivec2(1, 0)),
                (ivec3(0, 0, 1), ivec2(1, 1)),
            ],
            | Face::Front => [
                (ivec3(0, 1, 0), ivec2(0, 0)),
                (ivec3(0, 0, 0), ivec2(0, 1)),
                (ivec3(1, 1, 0), ivec2(1, 0)),
                (ivec3(1, 0, 0), ivec2(1, 1)),
            ],
        }
    }
}

#[derive(bon::Builder, Debug)]
pub struct Quad {
    position: glam::IVec3,
    block: block::Block,
    face: Face,
}

impl Quad {
    pub fn positions(&self) -> [glam::Vec3; 4] {
        self.face.corners().map(|(offset, _)| (self.position + offset).as_vec3())
    }

    pub fn texture_uvs(&self) -> [glam::Vec2; 4] {
        self.face.corners().map(|(_, uv)| uv.as_vec2())
    }

    pub fn normals(&self) -> [glam::Vec3; 4] {
        [self.face.normal(); 4]
    }

    fn indices(&self, start: u16) -> [u16; 6] {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }

    pub const CUBE: [Quad; 6] = [
        Quad {
            position: glam::IVec3::ZERO,
            block: block::Block::Air,
            face: Face::Top,
        },
        Quad {
            position: glam::IVec3::ZERO,
            block: block::Block::Air,
            face: Face::Bottom,
        },
        Quad {
            position: glam::IVec3::ZERO,
            block: block::Block::Air,
            face: Face::Left,
        },
        Quad {
            position: glam::IVec3::ZERO,
            block: block::Block::Air,
            face: Face::Right,
        },
        Quad {
            position: glam::IVec3::ZERO,
            block: block::Block::Air,
            face: Face::Front,
        },
        Quad {
            position: glam::IVec3::ZERO,
            block: block::Block::Air,
            face: Face::Back,
        },
    ];
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct TerrainVertex {
    pub pos: glam::Vec3,
    pub nor: glam::Vec3,
    pub tex: glam::Vec2,
}

impl render::GfxVertex for TerrainVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: &[wgpu::VertexAttribute] = &vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
        ];

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBS,
        }
    }
}

#[derive(bon::Builder, Debug, Default)]
pub struct RectilinearMesh {
    pub positions: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub tex_uvs: Vec<glam::Vec2>,
    pub indices: Vec<u16>,
    pub faces: Vec<Face>,
}

impl RectilinearMesh {
    pub fn from_quads(quads: &[Quad]) -> Self {
        let mut out = Self::default();
        for quad in quads {
            let len = out.positions.len();
            out.positions.extend_from_slice(&quad.positions());
            out.normals.extend_from_slice(&quad.normals());
            out.tex_uvs.extend_from_slice(&quad.texture_uvs());
            out.indices.extend_from_slice(&quad.indices(len as u16));
            out.faces.push(quad.face);
        }
        out
    }

    pub fn scale(&mut self, scale: glam::Vec3) {
        self.positions.iter_mut().for_each(|pos| {
            *pos *= scale;
        });
    }

    pub fn shift(&mut self, shift: glam::Vec3) {
        self.positions.iter_mut().for_each(|pos| {
            *pos += shift;
        });
    }

    pub fn unit_cube() -> Self {
        Self::from_quads(&Quad::CUBE)
    }
}

pub fn mesh_quads(
    context: &render::GfxContext,
    atlas: &atlas::TextureAtlas,
    quads: &[Quad],
) -> mesh::GfxMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    quads.iter().for_each(|quad| {
        let len = vertices.len();

        let pos = quad.positions();
        let nor = quad.normals();
        let mut uvs = quad.texture_uvs();
        atlas.conform_uvs(&mut uvs, &quad.block.to_string(), quad.face);
        (0..4).for_each(|idx| {
            vertices.push(TerrainVertex { pos: pos[idx], nor: nor[idx], tex: uvs[idx] });
        });

        let ind = quad.indices(len as u16);
        indices.extend_from_slice(&ind);
    });

    mesh::GfxMesh::new(context, &vertices, &indices)
}

pub fn mesh_chunk(
    context: &render::GfxContext,
    atlas: &atlas::TextureAtlas,
    chunk: &chunk::Chunk,
) -> mesh::GfxMesh {
    let mut quads = Vec::new();
    let origin_shift = chunk.offset * glam::ivec3(chunk.width as i32, 0, chunk.width as i32);
    for z in 0..chunk.width {
        for y in 0..chunk.height {
            for x in 0..chunk.width {
                let block = chunk.blocks.get([x, y, z]);

                if block == &block::Block::Air {
                    continue;
                }

                for face in Face::ALL {
                    let offset = face.neighbor_offset();
                    let neighbor = chunk.blocks.try_get([
                        (x as i32 + offset.x) as usize,
                        (y as i32 + offset.y) as usize,
                        (z as i32 + offset.z) as usize,
                    ]);

                    if let Some(neighbor) = neighbor
                        && neighbor != &block::Block::Air
                    {
                        continue;
                    }

                    quads.push(Quad {
                        position: glam::ivec3(x as i32, y as i32, z as i32) + origin_shift,
                        block: *block,
                        face,
                    });
                }
            }
        }
    }

    mesh_quads(context, atlas, &quads)
}

pub fn make_block_texture_checker(
    context: &render::GfxContext,
    atlas: &atlas::TextureAtlas,
) -> mesh::GfxMesh {
    let mut quads = Vec::new();
    Face::ALL.iter().for_each(|&quad_face| {
        quads.push(Quad {
            position: glam::ivec3(1, 1, 1),
            face: quad_face,
            block: block::Block::Water,
        });
        quads.push(Quad {
            position: glam::ivec3(1, 1, 2),
            face: quad_face,
            block: block::Block::Grass,
        });
        quads.push(Quad {
            position: glam::ivec3(1, 1, 3),
            face: quad_face,
            block: block::Block::Sand,
        });
        quads.push(Quad {
            position: glam::ivec3(1, 1, 4),
            face: quad_face,
            block: block::Block::Log,
        });
        quads.push(Quad {
            position: glam::ivec3(1, 2, 4),
            face: quad_face,
            block: block::Block::Leaf,
        });
    });

    mesh_quads(context, atlas, &quads)
}
