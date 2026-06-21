pub mod mesh;
pub mod resource;
pub mod util;

use std::sync;

use rustc_hash as rh;
use winit::window;

pub trait GfxVertex
where
     Self: bytemuck::Pod,
{
     fn descriptor() -> wgpu::VertexBufferLayout<'static>;
}

pub trait GfxCamera
{
     fn view_proj(&self) -> glam::Mat4;
}

pub trait GfxTransform
{
     fn model(&self) -> glam::Mat4;
}

pub trait GfxPipeline
{
     fn pipeline(context: &GfxContext, layouts: &[Option<&wgpu::BindGroupLayout>]) -> wgpu::RenderPipeline;
}

#[derive(bon::Builder, Debug)]
pub struct GfxContext
{
     pub surface: wgpu::Surface<'static>,
     pub config: wgpu::SurfaceConfiguration,
     pub device: wgpu::Device,
     pub queue: wgpu::Queue,
}

impl GfxContext
{
     pub async fn new(window: sync::Arc<window::Window>) -> anyhow::Result<Self>
     {
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
                    required_features: wgpu::Features::POLYGON_MODE_LINE
                         | wgpu::Features::CONSERVATIVE_RASTERIZATION
                         | wgpu::Features::default(),
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
          log::warn!("Surface format: {:?}", surface_format);
          let config = wgpu::SurfaceConfiguration {
               usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
               format: surface_format,
               width: window.inner_size().width,
               height: window.inner_size().height,
               present_mode: surface_caps.present_modes[0],
               // present_mode: wgpu::PresentMode::AutoNoVsync,
               desired_maximum_frame_latency: 2,
               alpha_mode: surface_caps.alpha_modes[0],
               view_formats: Vec::new(),
          };
          surface.configure(&device, &config);

          Ok(Self {
               surface,
               config,
               device,
               queue,
          })
     }

     pub fn config_changed(&mut self, width: u32, height: u32)
     {
          self.config.width = width;
          self.config.height = height;
          self.surface.configure(&self.device, &self.config);
     }
}

#[derive(bon::Builder, Debug)]
pub struct GfxDrawCall
{
     pub mesh: String,
     pub pipe: String,
     pub bind_groups: Vec<String>,
}

#[derive(bon::Builder, Debug, Default)]
pub struct GfxRenderer
{
     #[builder(default)]
     pub bind_group_layouts: rh::FxHashMap<String, wgpu::BindGroupLayout>,

     #[builder(default)]
     pub bind_groups: rh::FxHashMap<String, wgpu::BindGroup>,

     #[builder(default)]
     pub pipelines: rh::FxHashMap<String, wgpu::RenderPipeline>,

     #[builder(default)]
     pub meshes: rh::FxHashMap<String, mesh::GfxMesh>,

     #[builder(default)]
     pub resources: rh::FxHashMap<String, resource::GfxResource>,

     pub depth_texture: Option<resource::GfxTexture>,

     #[builder(default)]
     pub render_queue: Vec<GfxDrawCall>,
}

impl GfxRenderer
{
     pub fn new(_: &GfxContext) -> anyhow::Result<Self>
     {
          Ok(Self::builder().build())
     }

     pub fn config_changed(&mut self, context: &GfxContext) -> anyhow::Result<()>
     {
          self.depth_texture = Some(resource::GfxTexture::new_depth(context, "Main depth")?);
          Ok(())
     }

     pub fn register_bind_group_layout(
          &mut self,
          context: &GfxContext,
          name: &str,
          layouts: &[resource::GfxBindingLayout],
     ) -> anyhow::Result<()>
     {
          if self.bind_group_layouts.contains_key(name)
          {
               log::error!("Layout name already registered: {}", name);
               anyhow::bail!("Taken layouts: {:?}", self.bind_group_layouts);
          }

          let entries = layouts
               .iter()
               .enumerate()
               .map(|(index, layout)| layout.get_bind_group_layout(index as u32))
               .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

          let layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
               label: Some(&format!("{} bind group layout", name)),
               entries: &entries,
          });

          self.bind_group_layouts.insert(name.into(), layout);
          Ok(())
     }

     pub fn register_bind_group(
          &mut self,
          context: &GfxContext,
          name: &str,
          layout_name: &str,
          resource_names: &[&str],
     ) -> anyhow::Result<()>
     {
          let Some(layout) = self.bind_group_layouts.get(layout_name)
          else
          {
               log::error!("Layout must be registered first: {}", layout_name);
               anyhow::bail!("Avaliable layouts: {:?}", self.bind_group_layouts);
          };

          let entries = resource_names
               .iter()
               .enumerate()
               .map(|(index, &resource_name)| self.resources[resource_name].get_bind_group(index as u32))
               .collect::<Vec<wgpu::BindGroupEntry>>();

          let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
               label: Some(&format!("{} bind group", name)),
               layout,
               entries: &entries,
          });

          self.bind_groups.insert(name.into(), bind_group);
          Ok(())
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

     pub fn register_mesh(&mut self, name: &str, mesh: mesh::GfxMesh)
     {
          self.meshes.insert(name.into(), mesh);
     }

     pub fn register_resource(&mut self, name: &str, resource: resource::GfxResource)
     {
          self.resources.insert(name.into(), resource);
     }

     pub fn unregister_bind_group(&mut self, name: &str)
     {
          self.bind_groups.remove(name);
     }

     pub fn unregister_mesh(&mut self, name: &str)
     {
          self.meshes.remove(name);
     }

     pub fn unregister_resource(&mut self, name: &str)
     {
          self.resources.remove(name);
     }

     pub fn queue(&mut self, call: GfxDrawCall)
     {
          self.render_queue.push(call);
     }

     pub fn render(&mut self, render_pass: &mut wgpu::RenderPass)
     {
          let mut rc = GfxDrawCall {
               mesh: String::new(),
               pipe: String::new(),
               bind_groups: Vec::new(),
          };

          for call in self.render_queue.drain(..)
          {
               let Some(pipe) = self.pipelines.get(&call.pipe)
               else
               {
                    log::error!("Pipeline not found on call: {:?}", call);
                    continue;
               };

               let Some(mesh) = self.meshes.get(&call.mesh)
               else
               {
                    log::error!("Mesh not found on call: {:?}", call);
                    continue;
               };

               if mesh.size == 0
               {
                    continue;
               }

               if rc.pipe != call.pipe
               {
                    render_pass.set_pipeline(pipe);
               }

               if rc.mesh != call.mesh
               {
                    render_pass.set_vertex_buffer(0, mesh.vertex.slice(..));
                    render_pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint32);
               }

               for (index, bind_group) in call.bind_groups.iter().enumerate()
               {
                    let Some(bind_group) = self.bind_groups.get(bind_group)
                    else
                    {
                         log::error!("Bind group not found on call: {:?}", call);
                         continue;
                    };

                    if rc.bind_groups != call.bind_groups
                    {
                         render_pass.set_bind_group(index as u32, bind_group, &[]);
                    }
               }

               render_pass.draw_indexed(0 .. mesh.size, 0, 0 .. 1);

               rc = call;
          }
     }
}
