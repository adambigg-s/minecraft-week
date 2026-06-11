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
    fn pipeline(context: &GfxContext, layouts: &[Option<&wgpu::BindGroupLayout>]) -> wgpu::RenderPipeline;
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
pub struct GfxDrawCall {
    pub mesh: String,
    pub pipe: String,
    pub bind_groups: Vec<String>,
}

#[derive(bon::Builder, Debug, Default)]
pub struct GfxRenderer {
    pub bind_group_layouts: collections::HashMap<String, wgpu::BindGroupLayout>,
    pub bind_groups: collections::HashMap<String, wgpu::BindGroup>,
    pub pipelines: collections::HashMap<String, wgpu::RenderPipeline>,
    pub meshes: collections::HashMap<String, mesh::GfxMesh>,
    pub resources: collections::HashMap<String, resources::GfxResource>,
    pub render_queue: Vec<GfxDrawCall>,
}

impl GfxRenderer {
    pub fn new(_: &GfxContext) -> anyhow::Result<Self> {
        Ok(Self::default())
    }

    pub fn register_bind_group_layout(
        &mut self,
        context: &GfxContext,
        name: &str,
        layouts: &[resources::GfxBindingLayout],
    ) {
        let entries = layouts
            .iter()
            .enumerate()
            .map(|(index, layout)| wgpu::BindGroupLayoutEntry {
                binding: index as u32,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: match layout {
                    | resources::GfxBindingLayout::Uniform => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    | resources::GfxBindingLayout::Texture => wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    | resources::GfxBindingLayout::Sampler => {
                        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering)
                    }
                },
                count: None,
            })
            .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

        let layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} bind group layout", name)),
            entries: &entries,
        });

        self.bind_group_layouts.insert(name.into(), layout);
    }

    pub fn register_bind_group(
        &mut self,
        context: &GfxContext,
        name: &str,
        layout_name: &str,
        bindings: &[resources::GfxBindingData],
    ) {
        let layout = self.bind_group_layouts.get(layout_name).unwrap_or_else(|| {
            log::error!("Layout must be registered first: {}", layout_name);
            log::warn!("Avaliable layouts: {:?}", self.bind_group_layouts);
            panic!();
        });

        let entries = bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| wgpu::BindGroupEntry {
                binding: index as u32,
                resource: match binding {
                    | resources::GfxBindingData::Uniform(gfx_uniform) => {
                        wgpu::BindingResource::Buffer(gfx_uniform.buffer.as_entire_buffer_binding())
                    }
                    | resources::GfxBindingData::Texture(gfx_texture) => {
                        wgpu::BindingResource::TextureView(&gfx_texture.view)
                    }
                    | resources::GfxBindingData::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
                },
            })
            .collect::<Vec<wgpu::BindGroupEntry>>();

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} bind group", name)),
            layout,
            entries: &entries,
        });

        self.bind_groups.insert(name.into(), bind_group);
    }

    pub fn register_pipeline<Pipe>(&mut self, context: &GfxContext, name: &str, layout_names: &[&str])
    where
        Pipe: GfxPipeline,
    {
        let layouts = layout_names
            .iter()
            .map(|&name| self.bind_group_layouts.get(name))
            .collect::<Vec<Option<&wgpu::BindGroupLayout>>>();

        self.pipelines.insert(name.into(), Pipe::pipeline(context, &layouts));
    }

    pub fn register_mesh(&mut self, name: &str, mesh: mesh::GfxMesh) {
        self.meshes.insert(name.into(), mesh);
    }

    pub fn register_resource(&mut self, name: &str, resource: resources::GfxResource) {
        self.resources.insert(name.into(), resource);
    }

    pub fn queue(&mut self, call: GfxDrawCall) {
        self.render_queue.push(call);
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        let mut rc = GfxDrawCall {
            mesh: String::new(),
            pipe: String::new(),
            bind_groups: Vec::new(),
        };

        for call in self.render_queue.drain(..) {
            let Some(pipe) = self.pipelines.get(&call.pipe)
            else {
                log::error!("Pipeline not found on call: {:?}", call);
                continue;
            };

            let Some(mesh) = self.meshes.get(&call.mesh)
            else {
                log::error!("Mesh not found on call: {:?}", call);
                continue;
            };

            if rc.pipe != call.pipe {
                render_pass.set_pipeline(pipe);
            }

            if rc.mesh != call.mesh {
                render_pass.set_vertex_buffer(0, mesh.vertex.slice(..));
                render_pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint16);
            }

            for (index, bind_group) in call.bind_groups.iter().enumerate() {
                let Some(bind_group) = self.bind_groups.get(bind_group)
                else {
                    log::error!("Bind group not found on call: {:?}", call);
                    continue;
                };

                if let (Some(rc), Some(bg)) = (rc.bind_groups.get(index), call.bind_groups.get(index))
                    && rc != bg
                {
                    render_pass.set_bind_group(index as u32, bind_group, &[]);
                }
            }

            render_pass.draw_indexed(0..mesh.size, 0, 0..1);

            rc = call;
        }
    }
}
