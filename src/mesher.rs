use std::mem;

use wgpu::vertex_attr_array;

use crate::render::{self, mesh};

pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Back,
    Front,
}

impl Face {
    pub fn normal(&self) -> glam::Vec3 {
        match self {
            | Face::Top => glam::Vec3::Y,
            | Face::Bottom => glam::Vec3::NEG_Y,
            | Face::Left => glam::Vec3::X,
            | Face::Right => glam::Vec3::NEG_X,
            | Face::Back => glam::Vec3::Z,
            | Face::Front => glam::Vec3::NEG_Z,
        }
    }
}

pub enum Block {
    Air,
    Dirt,
}

pub struct Quad {
    location: glam::IVec3,
    face: Face,
    block: Block,
}

impl Quad {
    fn positions(&self) -> [glam::Vec3; 4] {
        let positions = match self.face {
            | Face::Top => [
                glam::ivec3(0, 1, 0),
                glam::ivec3(1, 1, 0),
                glam::ivec3(0, 1, 1),
                glam::ivec3(1, 1, 1),
            ],
            | Face::Bottom => [
                glam::ivec3(0, 0, 0),
                glam::ivec3(1, 0, 0),
                glam::ivec3(0, 0, 1),
                glam::ivec3(1, 0, 1),
            ],
            | Face::Left => [
                glam::ivec3(0, 0, 0),
                glam::ivec3(0, 1, 0),
                glam::ivec3(0, 0, 1),
                glam::ivec3(0, 1, 1),
            ],
            | Face::Right => [
                glam::ivec3(1, 0, 0),
                glam::ivec3(1, 1, 0),
                glam::ivec3(1, 0, 1),
                glam::ivec3(1, 1, 1),
            ],
            | Face::Back => [
                glam::ivec3(0, 0, 0),
                glam::ivec3(0, 1, 0),
                glam::ivec3(1, 0, 0),
                glam::ivec3(1, 1, 0),
            ],
            | Face::Front => [
                glam::ivec3(0, 0, 1),
                glam::ivec3(0, 1, 1),
                glam::ivec3(1, 0, 1),
                glam::ivec3(1, 1, 1),
            ],
        };

        [
            glam::vec3(
                self.location[0] as f32 + positions[0][0] as f32,
                self.location[1] as f32 + positions[0][1] as f32,
                self.location[2] as f32 + positions[0][2] as f32,
            ),
            glam::vec3(
                self.location[0] as f32 + positions[1][0] as f32,
                self.location[1] as f32 + positions[1][1] as f32,
                self.location[2] as f32 + positions[1][2] as f32,
            ),
            glam::vec3(
                self.location[0] as f32 + positions[2][0] as f32,
                self.location[1] as f32 + positions[2][1] as f32,
                self.location[2] as f32 + positions[2][2] as f32,
            ),
            glam::vec3(
                self.location[0] as f32 + positions[3][0] as f32,
                self.location[1] as f32 + positions[3][1] as f32,
                self.location[2] as f32 + positions[3][2] as f32,
            ),
        ]
    }

    pub fn normals(&self) -> [glam::Vec3; 4] {
        [self.face.normal(); 4]
    }

    pub fn texture_uvs(&self) -> [glam::Vec2; 4] {
        [
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 0.0),
            glam::vec2(0.0, 1.0),
            glam::vec2(1.0, 1.0),
        ]
    }

    fn indices(&self, start: u16) -> [u16; 6] {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }
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

pub fn mesh_quads(context: &render::GfxContext, quads: &[Quad]) -> mesh::GfxMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for quad in quads {
        let base = vertices.len();
        let positions = quad.positions();
        let normals = quad.normals();
        let uvs = quad.texture_uvs();

        for i in 0..4 {
            vertices.push(TerrainVertex { pos: positions[i], nor: normals[i], tex: uvs[i] });
        }
        indices.extend_from_slice(&quad.indices(base as u16));
    }

    mesh::GfxMesh::new(context, &vertices, &indices)
}

pub fn make_cube_mesh(context: &render::GfxContext) -> mesh::GfxMesh {
    let quads = vec![
        Quad {
            location: glam::IVec3::ZERO,
            face: Face::Top,
            block: Block::Dirt,
        },
        Quad {
            location: glam::IVec3::ZERO,
            face: Face::Bottom,
            block: Block::Dirt,
        },
        Quad {
            location: glam::IVec3::ZERO,
            face: Face::Left,
            block: Block::Dirt,
        },
        Quad {
            location: glam::IVec3::ZERO,
            face: Face::Right,
            block: Block::Dirt,
        },
        Quad {
            location: glam::IVec3::ZERO,
            face: Face::Front,
            block: Block::Dirt,
        },
        Quad {
            location: glam::IVec3::ZERO,
            face: Face::Back,
            block: Block::Dirt,
        },
    ];

    mesh_quads(context, &quads)
}
