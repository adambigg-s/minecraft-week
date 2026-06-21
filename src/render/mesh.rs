use wgpu::util::DeviceExt;
use wgpu::util::{self};

use crate::render;

#[derive(bon::Builder, Debug)]
pub struct GfxMesh
{
     pub vertex: wgpu::Buffer,
     pub index: wgpu::Buffer,
     pub size: u32,
}

impl GfxMesh
{
     pub fn new<Vertex, Index>(context: &render::GfxContext, vertices: &[Vertex], indices: &[Index]) -> Self
     where
          Vertex: render::GfxVertex,
          Index: Into<u32> + Copy,
     {
          let vertex = context.device.create_buffer_init(&util::BufferInitDescriptor {
               label: Some("Vertex buffer"),
               contents: bytemuck::cast_slice(vertices),
               usage: wgpu::BufferUsages::VERTEX,
          });
          let index = context.device.create_buffer_init(&util::BufferInitDescriptor {
               label: Some("Index buffer"),
               contents: bytemuck::cast_slice(
                    &indices.iter().map(|&index| index.into()).collect::<Vec<u32>>(),
               ),
               usage: wgpu::BufferUsages::INDEX,
          });
          let size = indices.len() as u32;

          Self {
               vertex,
               index,
               size,
          }
     }

     pub fn write<Vertex, Index>(
          &mut self,
          context: &render::GfxContext,
          vertices: &[Vertex],
          indices: &[Index],
     ) where
          Vertex: render::GfxVertex,
          Index: Into<u32> + Copy,
     {
          if self.size >= indices.len() as u32
          {
               context.queue.write_buffer(&self.vertex, 0, bytemuck::cast_slice(vertices));
               context.queue.write_buffer(
                    &self.index,
                    0,
                    bytemuck::cast_slice(&indices.iter().map(|&index| index.into()).collect::<Vec<u32>>()),
               );
               return;
          }
          *self = Self::new(context, vertices, indices);
     }
}
