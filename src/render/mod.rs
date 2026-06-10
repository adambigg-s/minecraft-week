pub mod mesh;
pub mod resources;

use std::{collections, sync};

use winit::window;

pub trait GfxVertex
where
    Self: bytemuck::Pod,
{
    fn descriptor() -> wgpu::VertexBufferLayout<'static>;
}

pub trait GfxCamera {
    fn view_proj(&self) -> glam::Mat4;
}

pub trait GfxTransform {
    fn model(&self) -> glam::Mat4;
}

pub trait GfxPipeline {
    fn pipeline(
        gfx_context: &GfxContext,
        gfx_layouts: &[Option<wgpu::BindGroupLayout>],
    ) -> wgpu::RenderPipeline;
}

#[derive(bon::Builder, Debug)]
pub struct GfxContext {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GfxContext {
    pub async fn new(window: sync::Arc<window::Window>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::debugging(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance.create_surface(sync::Arc::clone(&window))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;
        log::info!("Adapter information: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Main GPU device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;
        log::warn!("Device created: {}", device.adapter_info().name);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
        };
        surface.configure(&device, &config);

        Ok(Self { surface, config, device, queue })
    }
}

#[derive(bon::Builder, Debug)]
pub struct GfxDrawCall<'d> {
    pub mesh: &'d str,
    pub pipe: &'d str,
    pub bind_groups: Vec<&'d str>,
}

#[derive(bon::Builder, Debug, Default)]
pub struct GfxRenderer {
    pub bind_group_layouts: collections::HashMap<String, wgpu::BindGroupLayout>,
    pub bind_groups: collections::HashMap<String, wgpu::BindGroup>,
    pub pipelines: collections::HashMap<String, wgpu::RenderPipeline>,
    pub meshes: collections::HashMap<String, mesh::GfxMesh>,
    pub resources: collections::HashMap<String, resources::GfxResource>,
    pub render_queue: Vec<GfxDrawCall<'static>>,
}

impl GfxRenderer {
    pub fn new(_: &GfxContext) -> anyhow::Result<Self> {
        Ok(Self::default())
    }

    pub fn register_bind_group_layout(
        &mut self,
        context: &GfxContext,
        name: &str,
        layout: &[resources::GfxBindingLayout],
    ) {
    }

    pub fn register_bind_group(
        &mut self,
        context: &GfxContext,
        name: &str,
        layout_name: &str,
        bindings: &[resources::GfxBindingData],
    ) {
    }

    pub fn register_pipeline<Pipe>(&mut self, context: &GfxContext, name: &str, layout_names: &[&str])
    where
        Pipe: GfxPipeline,
    {
    }

    pub fn register_mesh(&mut self, name: &str, mesh: mesh::GfxMesh) {}

    pub fn register_resource(&mut self, name: &str, resource: resources::GfxResource) {}

    pub fn queue(&mut self, call: GfxDrawCall) {}

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {}
}
