use std::env;
use std::range;
use std::sync;
use std::time;

use crate::application::input;
use crate::application::{self};
use crate::engine::aabb;
use crate::engine::camera;
use crate::engine::kinematics::{self};
use crate::engine::player;
use crate::engine::ray::Cast;
use crate::engine::ray::{self};
use crate::engine::transform;
use crate::render::GfxCamera;
use crate::render::resource;
use crate::render::util;
use crate::render::{self};
use crate::terrain;
use crate::visual::atlas;
use crate::visual::pipelines;
use crate::visual::skybox;
use crate::world::block;
use crate::world::manager;

#[derive(bon::Builder, Debug)]
pub struct GfxConfiguration
{
     pub pipeline: String,
     pub avaliable_pipes: Vec<String>,
     pub ao_strength: f32,
}

#[derive(bon::Builder, Debug)]
pub struct FrameData
{
     pub dt: f32,
     pub time: f32,
     pub instant: time::Instant,
     pub tick: usize,
}

impl FrameData
{
     pub fn update(&mut self)
     {
          self.dt = self.instant.elapsed().as_secs_f32();
          self.time += self.dt;
          self.instant = time::Instant::now();
          self.tick += 1;
     }
}

#[derive(bon::Builder, Debug)]
pub struct MinecraftWeek
{
     pub camera: camera::Camera,
     pub player: player::PlayerController,
     pub world: manager::ChunkManager,
     pub gfx_config: GfxConfiguration,
     pub frame_data: FrameData,
     pub block_selection: usize,
}

impl application::Application for MinecraftWeek
{
     fn config() -> application::Config
     {
          application::Config::builder()
               .width(1920)
               .height(1080)
               .title("Another Minecraft clone game")
               .build()
     }

     fn setup(
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     ) -> anyhow::Result<Self>
     {
          let (context, render) = (gfx_context, gfx_render);

          let (texture_atlas, _) = register_resources(context, render)?;
          let texture_atlas = sync::Arc::new(texture_atlas);

          register_pipelines(context, render)?;

          register_bind_groups(context, render)?;

          configure_crosshair(context, render)?;

          let seed = env::args()
               .collect::<Vec<String>>()
               .get(1)
               .map(|val| val.parse::<u32>().unwrap_or(1))
               .unwrap_or(1);
          let terrain_gen = sync::Arc::new(terrain::TerrainGenerator::new(seed));

          let camera = camera::Camera::builder()
               .inner(transform::Transform::from_position(glam::vec3(0.0, 116.0, 0.0)))
               .fov(85.0)
               .znear(0.1)
               .zfear(1000.0)
               .build();

          let player = player::PlayerController::builder()
               .movespeed(8.0)
               .lookspeed(0.001)
               .collider(aabb::AaBb::point_sides(camera.inner.position.to_array(), [0.45, 0.85, 0.45]))
               .kinematics(kinematics::Kinematics::builder().up(glam::Vec3::Y).build())
               .build();

          let mut world = manager::ChunkManager::builder()
               .atlas(sync::Arc::clone(&texture_atlas))
               .view_distance(24)
               .terrain(sync::Arc::clone(&terrain_gen))
               .chunk_width(16)
               .chunk_height(256)
               .build();
          world.spawn_workers(3);

          let gfx_config = GfxConfiguration {
               pipeline: "terrain_pipe".into(),
               avaliable_pipes: vec![
                    "terrain_pipe".into(),
                    "wireframe_pipe".into(),
                    "time_gradient_pipe".into(),
                    "illumination_pipe".into(),
               ],
               ao_strength: 2.0,
          };

          let instant = time::Instant::now();
          let frame_data = FrameData {
               dt: 0.0,
               time: instant.elapsed().as_secs_f32(),
               instant: time::Instant::now(),
               tick: 0,
          };

          let block_selection = block::Block::Torch as usize;

          Ok(Self {
               camera,
               player,
               world,
               gfx_config,
               frame_data,
               block_selection,
          })
     }

     fn physics_frame(
          &mut self,
          input: &mut input::Input,
          gfx_context: &render::GfxContext,
          gfx_render: &render::GfxRenderer,
     )
     {
          let (_, _) = (gfx_context, gfx_render);

          self.frame_data.update();

          self.handle_logistics_input(input);
          self.handle_movement_input(input);
          self.handle_interaction_input(input);

          self.world.update_chunks(self.camera.inner.position, self.frame_data.time);
     }

     fn gfx_frame(
          &mut self,
          _: &input::Input,
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     )
     {
          let (context, render) = (gfx_context, gfx_render);

          self.camera.ar = context.config.width as f32 / context.config.height as f32;

          self.world.sync_gfx_chunks(context, render);

          self.update_resources(context, render);

          render.queue(render::GfxDrawCall {
               mesh: "crosshair_mesh".into(),
               pipe: "crosshair_pipe".into(),
               bind_groups: vec![],
          });

          if &self.gfx_config.pipeline == "terrain_pipe"
          {
               // render.queue(render::GfxDrawCall {
               //      mesh: "skybox_mesh".into(),
               //      pipe: "skybox_pipe".into(),
               //      bind_groups: vec!["global_bg".into(), "skybox_bg".into()],
               // });
               self.world.render_chunks.iter().for_each(|&coord| {
                    render.queue(render::GfxDrawCall {
                         mesh: manager::ChunkManager::chunk_key(coord),
                         pipe: self.gfx_config.pipeline.to_owned(),
                         bind_groups: vec!["global_bg".into()],
                    });
               });
          }
          else if &self.gfx_config.pipeline == "time_gradient_pipe"
          {
               self.world.render_chunks.iter().for_each(|&coord| {
                    render.queue(render::GfxDrawCall {
                         mesh: manager::ChunkManager::chunk_key(coord),
                         pipe: self.gfx_config.pipeline.to_owned(),
                         bind_groups: vec![
                              "global_bg".into(),
                              format!("{}_time_bg", manager::ChunkManager::chunk_key(coord)),
                         ],
                    });
               });
          }
          else
          {
               self.world.render_chunks.iter().for_each(|&coord| {
                    render.queue(render::GfxDrawCall {
                         mesh: manager::ChunkManager::chunk_key(coord),
                         pipe: self.gfx_config.pipeline.to_owned(),
                         bind_groups: vec!["global_bg".into()],
                    });
               });
          }
     }
}

impl MinecraftWeek
{
     fn handle_logistics_input(&mut self, input: &mut input::Input)
     {
          if input.consume_key_press("escape")
          {
               input.request_quit = !input.request_quit;
               self.world.request_shutdown();
          }
          if input.consume_key_release("keyq")
          {
               input.request_grab = !input.request_grab;
          }
          if input.consume_key_press("keyr")
          {
               for (index, pipe) in self.gfx_config.avaliable_pipes.iter().enumerate()
               {
                    if pipe == &self.gfx_config.pipeline
                    {
                         self.gfx_config.pipeline = self.gfx_config.avaliable_pipes
                              [(index + 1) % self.gfx_config.avaliable_pipes.len()]
                         .to_owned();
                         break;
                    }
               }
          }
          if input.consume_key_press("equal")
          {
               self.world.view_distance = self.world.view_distance.saturating_add(1);
          }
          if input.consume_key_press("minus")
          {
               self.world.view_distance = self.world.view_distance.saturating_sub(1);
          }
          if input.consume_key_release("keyy")
          {
               self.player.collisions = !self.player.collisions;
          }
          if input.consume_key_press("keyf")
          {
               self.block_selection = self.block_selection.wrapping_add(1);
               log::info!(
                    "Block selection: {}",
                    block::Block::from(self.block_selection as u8 % block::Block::BlockCounter as u8)
               );
          }
          if input.consume_key_press("keyg")
          {
               self.block_selection = self.block_selection.wrapping_sub(1);
               log::info!(
                    "Block selection: {}",
                    block::Block::from(self.block_selection as u8 % block::Block::BlockCounter as u8)
               );
          }
          if input.consume_key_press("keyl")
          {
               self.block_selection = block::Block::Light as usize;
          }
          if input.consume_key_press("keyo")
          {
               self.gfx_config.ao_strength -= 0.5;
               log::info!("AO strength: {}", self.gfx_config.ao_strength);
          }
          if input.consume_key_press("keyp")
          {
               self.gfx_config.ao_strength += 0.5;
               log::info!("AO strength: {}", self.gfx_config.ao_strength);
          }
     }

     fn handle_movement_input(&mut self, input: &mut input::Input)
     {
          if input.consume_key_press("digit1")
          {
               self.player.movespeed *= 0.5;
          }
          if input.consume_key_press("digit2")
          {
               self.player.movespeed *= 2.0;
          }

          let [mut dy, mut dx] = input.consume_mouse_delta().into();
          [dy, dx] = (glam::vec2(dy, dx) * self.player.lookspeed).to_array();
          self.camera.yaw -= dy;
          self.camera.pitch -= dx;
          self.camera.confine_euler();
          self.camera.inner.rotation = glam::Quat::from_rotation_z(0.0)
               * glam::Quat::from_rotation_y(self.camera.yaw)
               * glam::Quat::from_rotation_x(self.camera.pitch);

          match self.player.collisions
          {
               | true =>
               {
                    let mut frame_movement_speed = self.player.movespeed;
                    let [mut dx, _, mut dz] = [0.0; 3];
                    if input.get_key_pres("keyw")
                    {
                         dz += 1.0;
                    }
                    if input.get_key_pres("keys")
                    {
                         dz -= 1.0;
                    }
                    if input.get_key_pres("keyd")
                    {
                         dx += 1.0;
                    }
                    if input.get_key_pres("keya")
                    {
                         dx -= 1.0;
                    }
                    if input.get_key_pres("space")
                    {
                         self.player.kinematics.jump(9.5);
                    }
                    if input.get_key_pres("shiftleft")
                    {
                         frame_movement_speed *= 1.5;
                    }
                    let forward = self.camera.inner.forward().with_y(0.0).normalize_or_zero();
                    let right = self.camera.inner.right().with_y(0.0).normalize_or_zero();
                    let movement = (right * dx + forward * dz).normalize_or_zero();
                    self.player.kinematics.velocity.x = movement.x * frame_movement_speed;
                    self.player.kinematics.velocity.z = movement.z * frame_movement_speed;
                    self.player.kinematics.apply_gravity(32.0, self.frame_data.dt);
                    self.player.collider = self.player.kinematics.translate(
                         self.player.collider,
                         &self.world,
                         self.frame_data.dt,
                    );
                    self.camera.inner.position = self.player.collider.center() + glam::vec3(0.0, 0.65, 0.0);
               }
               | false =>
               {
                    let [mut dx, mut dy, mut dz] = [0.0; 3];
                    if input.get_key_pres("keyw")
                    {
                         dz += 1.0;
                    }
                    if input.get_key_pres("keys")
                    {
                         dz -= 1.0;
                    }
                    if input.get_key_pres("keyd")
                    {
                         dx += 1.0;
                    }
                    if input.get_key_pres("keya")
                    {
                         dx -= 1.0;
                    }
                    if input.get_key_pres("space")
                    {
                         dy += 1.0;
                    }
                    if input.get_key_pres("shiftleft")
                    {
                         dy -= 1.0;
                    }
                    [dx, dy, dz] = (glam::vec3(dx, dy, dz).normalize_or_zero()
                         * self.player.movespeed
                         * self.frame_data.dt)
                         .to_array();
                    self.camera.update_position(dx, dy, dz);
                    self.player.collider =
                         self.player.collider + (self.camera.inner.position - self.player.collider.center());
               }
          }
     }

     fn update_resources(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer)
     {
          if let Some(resource::GfxResource::Uniform(cam_view_proj)) = render.resources.get("camera_uni")
          {
               cam_view_proj.write(context, &self.camera.view_proj());
          }
          if let Some(resource::GfxResource::Uniform(cam_view)) = render.resources.get("camera_view_uni")
          {
               cam_view.write(context, &self.camera.view());
          }
          if let Some(resource::GfxResource::Uniform(global_time)) = render.resources.get("time_uni")
          {
               global_time.write(context, &self.frame_data.time);
          }
          if let Some(resource::GfxResource::Uniform(ao_strength)) = render.resources.get("ao_uni")
          {
               ao_strength.write(context, &self.gfx_config.ao_strength);
          }

          for (&chunk_coord, time) in self.world.chunk_map.update_times.iter()
          {
               if let Some(resource::GfxResource::Uniform(chunk_time)) = render
                    .resources
                    .get(&format!("{}_time_uni", manager::ChunkManager::chunk_key(chunk_coord)))
               {
                    chunk_time.write(context, time);
               }
          }
     }

     fn handle_interaction_input(&mut self, input: &mut input::Input)
     {
          let ray = ray::Ray {
               origin: self.camera.inner.position,
               direction: self.camera.inner.forward().normalize(),
               tspan: range::Range {
                    start: 0.0,
                    end: 50.0,
               },
          };
          if input.consume_mouse_left_press()
               && let Some(hit) = self.world.cast(ray)
          {
               self.world.modify(hit.position, block::Block::Air);
          }
          if input.consume_mouse_right_press()
               && let Some(hit) = self.world.cast(ray)
          {
               self.world.modify(
                    hit.position + hit.normal,
                    block::Block::from(self.block_selection as u8 % block::Block::BlockCounter as u8),
               );
          }
     }
}

fn register_bind_groups(
     context: &mut render::GfxContext,
     render: &mut render::GfxRenderer,
) -> Result<(), anyhow::Error>
{
     render.register_bind_group(
          context,
          "global_bg",
          "global_layout",
          &[
               "camera_uni",
               "camera_view_uni",
               "texture_atlas",
               "sampler",
               "time_uni",
               "ao_uni",
          ],
     )?;
     render.register_bind_group(context, "skybox_bg", "skybox_layout", &["skybox_atlas", "sampler"])?;
     Ok(())
}

fn register_resources(
     context: &mut render::GfxContext,
     render: &mut render::GfxRenderer,
) -> Result<(atlas::TextureAtlas, skybox::Skybox), anyhow::Error>
{
     let skybox = create_skybox(context, render)?;
     let atlas = create_atlas()?;
     render.register_resource(
          "skybox_atlas",
          util::texture_image(context, &skybox.texture.atlas, "Skybox atlas"),
     );
     render.register_resource("texture_atlas", util::texture_image(context, &atlas.atlas, "Main atlas"));
     render.register_resource("sampler", util::sampler(context, "Sampler"));
     render.register_resource("camera_uni", util::uniform::<glam::Mat4>(context, "Camera"));
     render.register_resource("camera_view_uni", util::uniform::<glam::Mat4>(context, "Camera view"));
     render.register_resource("time_uni", util::uniform::<f32>(context, "Global time"));
     render.register_resource("ao_uni", util::uniform::<f32>(context, "Global AO strength"));
     Ok((atlas, skybox))
}

fn create_atlas() -> Result<atlas::TextureAtlas, anyhow::Error>
{
     let atlas = atlas::TextureAtlas::new("./res/blocks/", 16)?;
     atlas.save("./res/block_atlas.png")?;
     Ok(atlas)
}

fn create_skybox(
     context: &mut render::GfxContext,
     render: &mut render::GfxRenderer,
) -> Result<skybox::Skybox, anyhow::Error>
{
     let mut skybox = skybox::Skybox::new("./res/skybox/", 32, 500.0)?;
     skybox.texture.save("./res/skybox_atlas.png")?;
     render.register_mesh("skybox_mesh", skybox.create_gfx_mesh(context));
     Ok(skybox)
}

fn register_pipelines(
     context: &mut render::GfxContext,
     render: &mut render::GfxRenderer,
) -> Result<(), anyhow::Error>
{
     render.register_bind_group_layout(
          context,
          "global_layout",
          &[
               resource::GfxBindingLayout::Uniform,
               resource::GfxBindingLayout::Uniform,
               resource::GfxBindingLayout::Texture,
               resource::GfxBindingLayout::Sampler,
               resource::GfxBindingLayout::Uniform,
               resource::GfxBindingLayout::Uniform,
          ],
     )?;
     render.register_bind_group_layout(
          context,
          "skybox_layout",
          &[
               resource::GfxBindingLayout::Texture,
               resource::GfxBindingLayout::Sampler,
          ],
     )?;
     render.register_bind_group_layout(context, "time_layout", &[resource::GfxBindingLayout::Uniform])?;

     render.register_pipeline::<pipelines::Terrain>(context, "terrain_pipe", &["global_layout"]);
     render.register_pipeline::<pipelines::WireFrame>(context, "wireframe_pipe", &["global_layout"]);
     render.register_pipeline::<pipelines::CulledFrame>(context, "culledframe_pipe", &["global_layout"]);
     render.register_pipeline::<pipelines::TimeGradient>(
          context,
          "time_gradient_pipe",
          &["global_layout", "time_layout"],
     );
     render.register_pipeline::<pipelines::Skybox>(
          context,
          "skybox_pipe",
          &["global_layout", "skybox_layout"],
     );
     render.register_pipeline::<pipelines::Crosshair>(context, "crosshair_pipe", &[]);
     render.register_pipeline::<pipelines::BlockIllumination>(
          context,
          "illumination_pipe",
          &["global_layout"],
     );
     Ok(())
}

fn configure_crosshair(
     context: &mut render::GfxContext,
     render: &mut render::GfxRenderer,
) -> anyhow::Result<()>
{
     render.register_mesh(
          "crosshair_mesh",
          util::mesh(context, pipelines::TRI_VERTICES, pipelines::TRI_INDICES),
     );
     Ok(())
}
