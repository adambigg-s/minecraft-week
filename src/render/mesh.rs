use wgpu::util::{self, DeviceExt};

use crate::render;

#[derive(bon::Builder, Debug)]
pub struct GfxMesh {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub size: u32,
}

impl GfxMesh {
    pub fn new<Vertex, Index>(
        context: &render::GfxContext,
        vertices: &[Vertex],
        indices: &[Index],
        label: &str,
    ) -> Self
    where
        Vertex: render::GfxVertex,
        Index: Into<u16> + Copy,
    {
        let vertex = context.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some(&format!("{} vertex buffer", label)),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index = context.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some(&format!("{} index buffer", label)),
            contents: bytemuck::cast_slice(&indices.iter().map(|&index| index.into()).collect::<Vec<u16>>()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let size = indices.len() as u32;

        Self { vertex, index, size }
    }
}
