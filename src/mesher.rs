use std::mem;

use wgpu::vertex_attr_array;

use crate::{
    atlas, block,
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
    location: glam::IVec3,
    face: Face,
    block: block::Block,
}

impl Quad {
    pub fn positions(&self) -> [glam::Vec3; 4] {
        self.face.corners().map(|(offset, _)| (self.location + offset).as_vec3())
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

pub fn mesh_quads(
    context: &render::GfxContext,
    atlas: &atlas::TextureAtlas,
    quads: &[Quad],
) -> mesh::GfxMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for quad in quads {
        let base = vertices.len();
        let pos = quad.positions();
        let nor = quad.normals();
        let ind = quad.indices(base as u16);
        let mut uvs = quad.texture_uvs();
        atlas.conform_uvs(&mut uvs, &quad.block.to_string(), quad.face);

        for i in 0..4 {
            vertices.push(TerrainVertex { pos: pos[i], nor: nor[i], tex: uvs[i] });
        }
        indices.extend_from_slice(&ind);
    }

    mesh::GfxMesh::new(context, &vertices, &indices)
}

pub fn make_cube_mesh(context: &render::GfxContext, atlas: &atlas::TextureAtlas) -> mesh::GfxMesh {
    let faces = [
        Face::Top,
        Face::Bottom,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];
    let quads = faces.map(|face| Quad {
        location: glam::ivec3(1, 1, 1),
        face,
        block: block::Block::Sand,
    });

    mesh_quads(context, atlas, &quads)
}
