use std::mem;

use wgpu::vertex_attr_array;

use crate::{
    atlas, block, chunk,
    render::{self},
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
    face: Face,
}

impl Quad {
    pub fn cube() -> [Self; 6] {
        Face::ALL.map(|face| Self { position: glam::IVec3::ZERO, face })
    }

    pub fn positions(&self) -> [glam::Vec3; 4] {
        self.face.corners().map(|(offset, _)| (self.position + offset).as_vec3())
    }

    pub fn texture_uvs(&self) -> [glam::Vec2; 4] {
        self.face.corners().map(|(_, uv)| uv.as_vec2())
    }

    pub fn normals(&self) -> [glam::Vec3; 4] {
        [self.face.normal(); 4]
    }

    pub fn indices(&self, start: u32) -> [u32; 6] {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }
}

#[derive(bon::Builder, Debug)]
pub struct RectilinearMeshSlice<'r> {
    pub face: Face,
    pub integer_position: glam::IVec3,
    pub pos: &'r mut [glam::Vec3],
    pub nor: &'r mut [glam::Vec3],
    pub uvs: &'r mut [glam::Vec2],
}

#[derive(bon::Builder, Debug, Default)]
pub struct RectilinearMesh {
    pub positions: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub uvs: Vec<glam::Vec2>,
    pub indices: Vec<u32>,
    pub integer_positions: Vec<glam::IVec3>,
    pub faces: Vec<Face>,
    pub size: usize,
}

impl RectilinearMesh {
    pub fn from_quads(quads: &[Quad]) -> Self {
        let mut out = Self { size: quads.len(), ..Default::default() };
        quads.iter().for_each(|quad| {
            let len = out.positions.len();
            out.positions.extend_from_slice(&quad.positions());
            out.normals.extend_from_slice(&quad.normals());
            out.uvs.extend_from_slice(&quad.texture_uvs());
            out.indices.extend_from_slice(&quad.indices(len as u32));
            out.faces.push(quad.face);
            out.integer_positions.push(quad.position);
        });
        out
    }

    pub fn quad_slice<'r>(&'r mut self, index: usize) -> RectilinearMeshSlice<'r> {
        let offset = index * 4;
        RectilinearMeshSlice {
            face: self.faces[index],
            integer_position: self.integer_positions[index],
            pos: &mut self.positions[offset..offset + 4],
            nor: &mut self.normals[offset..offset + 4],
            uvs: &mut self.uvs[offset..offset + 4],
        }
    }

    pub fn unit_cube() -> Self {
        Self::from_quads(&Quad::cube())
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

pub struct ChunkMesher<'c> {
    pub chunk: &'c chunk::Chunk,
    pub atlas: &'c atlas::TextureAtlas,
}

impl<'c> ChunkMesher<'c> {
    pub fn to_rectilinear(&self) -> RectilinearMesh {
        let mut quads = Vec::new();
        let global = self.chunk.offset * glam::ivec3(self.chunk.width as i32, 0, self.chunk.width as i32);
        for z in 0..self.chunk.width {
            for y in 0..self.chunk.height {
                for x in 0..self.chunk.width {
                    let block = self.chunk.blocks.get([x, y, z]);

                    if block == &block::Block::Air {
                        continue;
                    }

                    for face in Face::ALL {
                        let offset = face.neighbor_offset();
                        let neighbor = self.chunk.blocks.try_get([
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
                            position: glam::ivec3(x as i32, y as i32, z as i32) + global,
                            face,
                        });
                    }
                }
            }
        }
        RectilinearMesh::from_quads(&quads)
    }

    pub fn map_uvs(&self, rectilinear: &mut RectilinearMesh) {
        (0..rectilinear.size).for_each(|index| {
            let RectilinearMeshSlice { face, integer_position, uvs, .. } = rectilinear.quad_slice(index);

            let position = self.chunk.to_chunk_coords(integer_position);
            let block = self.chunk.get(position);
            self.atlas.conform_uvs(uvs, block.name(), face);
        });
    }
}
