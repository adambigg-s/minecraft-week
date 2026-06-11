use std::mem;

use wgpu::vertex_attr_array;

use crate::render::{self, GfxVertex};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct Vertex {
    pos: glam::Vec3,
    col: glam::Vec3,
    tex: glam::Vec2,
}

impl render::GfxVertex for Vertex {
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

pub const TRI_INDICES: &[u16] = &[0, 1, 2];
pub const TRI_VERTICES: &[Vertex] = &[
    Vertex {
        pos: glam::vec3(-0.5, -0.5, 0.0),
        col: glam::vec3(1.0, 0.5, 0.0),
        tex: glam::vec2(0.0, 0.0),
    },
    Vertex {
        pos: glam::vec3(0.5, -0.5, 0.0),
        col: glam::vec3(0.0, 1.0, 0.5),
        tex: glam::vec2(0.0, 1.0),
    },
    Vertex {
        pos: glam::vec3(0.0, 0.5, 0.0),
        col: glam::vec3(0.5, 0.0, 1.0),
        tex: glam::vec2(1.0, 0.0),
    },
];

pub struct Rainbow;
impl render::GfxPipeline for Rainbow {
    fn pipeline(
        context: &render::GfxContext,
        layouts: &[Option<&wgpu::BindGroupLayout>],
    ) -> wgpu::RenderPipeline {
        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rainbow shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../src/shaders/rainbow.wgsl").into()),
        });

        let layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rainbow layout"),
            bind_group_layouts: layouts,
            immediate_size: 0,
        });

        context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rainbow pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::descriptor()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    }
}
