use std::{mem, sync};

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
    DiagPosFront,
    DiagPosBack,
    DiagNegFront,
    DiagNegBack,
}

impl Face {
    pub const CARDINALS: [Face; 6] = [
        Face::Top,
        Face::Bottom,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];
    pub const DIAGONALS: [Face; 4] = [
        Face::DiagPosFront,
        Face::DiagPosBack,
        Face::DiagNegFront,
        Face::DiagNegBack,
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
            | _ => glam::IVec3::ZERO,
        }
    }

    #[rustfmt::skip]
    pub fn normal(&self) -> glam::Vec3 {
        match self {
            | Face::Top          => glam::Vec3::Y,
            | Face::Bottom       => glam::Vec3::NEG_Y,
            | Face::Left         => glam::Vec3::X,
            | Face::Right        => glam::Vec3::NEG_X,
            | Face::Back         => glam::Vec3::Z,
            | Face::Front        => glam::Vec3::NEG_Z,
            | Face::DiagPosFront => glam::ivec3(1, 0, -1).as_vec3().normalize(),
            | Face::DiagPosBack  => glam::ivec3(1, 0, 1).as_vec3().normalize(),
            | Face::DiagNegFront => glam::ivec3(-1, 0, -1).as_vec3().normalize(),
            | Face::DiagNegBack  => glam::ivec3(-1, 0, 1).as_vec3().normalize(),
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
            | Face::DiagPosFront => [
                (ivec3(0, 1, 0), ivec2(0, 0)),
                (ivec3(0, 0, 0), ivec2(0, 1)),
                (ivec3(1, 1, 1), ivec2(1, 0)),
                (ivec3(1, 0, 1), ivec2(1, 1)),
            ],
            | Face::DiagPosBack => [
                (ivec3(1, 1, 1), ivec2(0, 0)),
                (ivec3(1, 0, 1), ivec2(0, 1)),
                (ivec3(0, 1, 0), ivec2(1, 0)),
                (ivec3(0, 0, 0), ivec2(1, 1)),
            ],
            | Face::DiagNegFront => [
                (ivec3(1, 1, 0), ivec2(0, 0)),
                (ivec3(1, 0, 0), ivec2(0, 1)),
                (ivec3(0, 1, 1), ivec2(1, 0)),
                (ivec3(0, 0, 1), ivec2(1, 1)),
            ],
            | Face::DiagNegBack => [
                (ivec3(0, 1, 1), ivec2(0, 0)),
                (ivec3(0, 0, 1), ivec2(0, 1)),
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
        Face::CARDINALS.map(|face| Self { position: glam::IVec3::ZERO, face })
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
    pub lum: f32,
    pub ao: f32,
}

impl render::GfxVertex for TerrainVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: &[wgpu::VertexAttribute] = &vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
            3 => Float32,
            4 => Float32,
        ];

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBS,
        }
    }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkRawMesh {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    pub offset: glam::IVec3,
}

#[derive(bon::Builder, Debug)]
pub struct ChunkMeshingAssisant {
    pub chunk: sync::Arc<chunk::Chunk>,
    pub neighbors: [Option<sync::Arc<chunk::Chunk>>; 4],
}

impl ChunkMeshingAssisant {
    pub const EMPTY: block::Block = block::Block::Air;
    pub const NORTH: usize = 0;
    pub const SOUTH: usize = 1;
    pub const EAST: usize = 2;
    pub const WEST: usize = 3;

    pub fn get_adjacent(&self, coord: glam::IVec3) -> block::Block {
        let width = (self.chunk.width - 1) as i32;
        let height = self.chunk.height as i32;

        if coord.y < 0 || coord.y >= height {
            return Self::EMPTY;
        }

        let target_chunk = if coord.z > width {
            &self.neighbors[Self::NORTH]
        }
        else if coord.z < 0 {
            &self.neighbors[Self::SOUTH]
        }
        else if coord.x < 0 {
            &self.neighbors[Self::EAST]
        }
        else if coord.x > width {
            &self.neighbors[Self::WEST]
        }
        else {
            return *self.chunk.get(coord);
        };

        let local_coord = self.chunk.to_chunk_coords(coord);
        match target_chunk {
            | Some(neighbor) => *neighbor.get(local_coord),
            | None => Self::EMPTY,
        }
    }
}

#[derive(bon::Builder, Debug)]
pub struct ChunkMesher<'c> {
    pub chunks: ChunkMeshingAssisant,
    pub atlas: &'c atlas::TextureAtlas,
}

impl<'c> ChunkMesher<'c> {
    pub fn to_rectilinear(&self) -> RectilinearMesh {
        use block::{Block::*, Visibility::*};

        let mut quads = Vec::new();
        let global = self.chunks.chunk.offset * self.chunks.chunk.size();

        for z in 0..self.chunks.chunk.width {
            for y in 0..self.chunks.chunk.height {
                for x in 0..self.chunks.chunk.width {
                    let block = self.chunks.chunk.blocks.get([x, y, z]);

                    if block == &Air {
                        continue;
                    }

                    for face in Face::CARDINALS {
                        let offset = face.neighbor_offset();
                        let neighbor = self.chunks.get_adjacent(glam::ivec3(
                            x as i32 + offset.x,
                            y as i32 + offset.y,
                            z as i32 + offset.z,
                        ));

                        let emit = match (block.visibility(), neighbor.visibility()) {
                            | (Opaque, PartialOpaque)
                            | (Opaque, Transparent)
                            | (Opaque, Invisible)
                            | (PartialOpaque, PartialOpaque)
                            | (PartialOpaque, Transparent)
                            | (PartialOpaque, Invisible)
                            | (Transparent, Invisible)
                            | (Transparent, PartialOpaque) => true,
                            | _ => false,
                        };
                        if !emit {
                            continue;
                        }

                        let position = glam::ivec3(x as i32, y as i32, z as i32) + global;
                        match block.mesh_style() {
                            | block::EmittedMesh::RectilinearFull => {
                                quads.push(Quad { position, face });
                            }
                            | block::EmittedMesh::Decorator => {
                                quads.extend(Face::DIAGONALS.map(|face| Quad { position, face }));
                            }
                            | block::EmittedMesh::RectilinearPartial => todo!(),
                        }
                    }
                }
            }
        }
        RectilinearMesh::from_quads(&quads)
    }

    pub fn map_uvs(&self, rectilinear: &mut RectilinearMesh) {
        (0..rectilinear.size).for_each(|index| {
            let RectilinearMeshSlice { face, integer_position, uvs, .. } = rectilinear.quad_slice(index);

            let position = self.chunks.chunk.to_chunk_coords(integer_position);
            let block = self.chunks.chunk.get(position);
            self.atlas.conform_uvs(uvs, block.name(), face);
        });
    }
}
