use std::{sync, time};

use crate::{
    application::{self, input},
    atlas, chunk,
    engine::{aabb, camera, transform},
    pipelines, player,
    render::{self, GfxCamera, resource, util},
    skybox, terrain,
};

#[derive(bon::Builder, Debug)]
pub struct MinecraftWeek {
    pub camera: camera::Camera,
    pub player: player::PlayerController,
    pub world: chunk::ChunkManager,
    pub pipeline: String,
    pub avaliable_pipelines: Vec<String>,
    pub tick: usize,
    pub time: f32,
    pub instant: time::Instant,
}

impl application::Application for MinecraftWeek {
    fn config() -> application::Config {
        application::Config::builder()
            .width(1920)
            .height(1080)
            .title("Minecraft-week game")
            .build()
    }

    fn setup(
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    ) -> anyhow::Result<Self> {
        let (context, render) = (gfx_context, gfx_render);

        let (texture_atlas, _) = register_resources(context, render)?;
        let texture_atlas = sync::Arc::new(texture_atlas);

        register_pipelines(context, render)?;

        register_bind_groups(context, render)?;

        let terrain_gen = sync::Arc::new(terrain::TerrainGenerator::new(1));

        let camera = camera::Camera::builder()
            .inner(transform::Transform::from_position(glam::vec3(0.0, 150.0, 0.0)))
            .fov(70.0)
            .znear(0.1)
            .zfear(1000.0)
            .build();
        let player = player::PlayerController::builder()
            .movespeed(0.5)
            .lookspeed(0.0025)
            .collider(aabb::AaBb::point_sides(camera.inner.position.to_array(), [0.5, 1.0, 0.5]))
            .build();

        let mut world = chunk::ChunkManager::builder()
            .atlas(sync::Arc::clone(&texture_atlas))
            .view_distance(18)
            .terrain(sync::Arc::clone(&terrain_gen))
            .chunk_width(32)
            .chunk_height(128)
            .build();
        world.spawn_worker();

        let pipeline = "terrain_pipe".into();
        let avaliable_pipelines = vec![
            "terrain_pipe".into(),
            "wireframe_pipe".into(),
            "culledframe_pipe".into(),
        ];
        let tick = 0;
        let instant = time::Instant::now();
        let time = instant.elapsed().as_secs_f32();

        Ok(Self {
            camera,
            player,
            world,
            pipeline,
            avaliable_pipelines,
            tick,
            time,
            instant,
        })
    }

    fn physics_frame(
        &mut self,
        input: &mut input::Input,
        gfx_context: &render::GfxContext,
        gfx_render: &render::GfxRenderer,
    ) {
        let (_, _) = (gfx_context, gfx_render);

        self.handle_logistics_input(input);
        self.handle_movement_input(input);

        self.world.update_chunks(self.camera.inner.position);
        self.tick += 1;
        self.time += self.instant.elapsed().as_secs_f32();
        self.instant = time::Instant::now();
    }

    fn gfx_frame(
        &mut self,
        _: &input::Input,
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    ) {
        let (context, render) = (gfx_context, gfx_render);

        self.camera.ar = context.config.width as f32 / context.config.height as f32;

        self.world.sync_gfx_chunks(context, render);

        self.update_resources(context, render);

        if &self.pipeline == "terrain_pipe" {
            render.queue(render::GfxDrawCall {
                mesh: "skybox_mesh".into(),
                pipe: "skybox_pipe".into(),
                bind_groups: vec!["global_bg".into(), "skybox_bg".into()],
            });
        }

        self.world.render_chunks.iter().for_each(|&coord| {
            render.queue(render::GfxDrawCall {
                mesh: chunk::ChunkManager::chunk_key(coord),
                pipe: self.pipeline.to_owned(),
                bind_groups: vec!["global_bg".into()],
            });
        });
    }
}

fn register_bind_groups(
    context: &mut render::GfxContext,
    render: &mut render::GfxRenderer,
) -> Result<(), anyhow::Error> {
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
        ],
    )?;
    render.register_bind_group(context, "skybox_bg", "skybox_layout", &["skybox_atlas", "sampler"])?;
    Ok(())
}

fn register_resources(
    context: &mut render::GfxContext,
    render: &mut render::GfxRenderer,
) -> Result<(atlas::TextureAtlas, skybox::Skybox), anyhow::Error> {
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
    Ok((atlas, skybox))
}

fn create_atlas() -> Result<atlas::TextureAtlas, anyhow::Error> {
    let atlas = atlas::TextureAtlas::new("./res/", 16)?;
    atlas.save("./res/atlas/atlas.png")?;
    Ok(atlas)
}

fn create_skybox(
    context: &mut render::GfxContext,
    render: &mut render::GfxRenderer,
) -> Result<skybox::Skybox, anyhow::Error> {
    let mut skybox = skybox::Skybox::new("./res/skybox/", 32, 500.0)?;
    skybox.texture.save("./res/atlas/skybox_atlas.png")?;
    render.register_mesh("skybox_mesh", skybox.create_gfx_mesh(context));
    Ok(skybox)
}

fn register_pipelines(
    context: &mut render::GfxContext,
    render: &mut render::GfxRenderer,
) -> Result<(), anyhow::Error> {
    render.register_bind_group_layout(
        context,
        "global_layout",
        &[
            resource::GfxBindingLayout::Uniform,
            resource::GfxBindingLayout::Uniform,
            resource::GfxBindingLayout::Texture,
            resource::GfxBindingLayout::Sampler,
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
    Ok(())
}

impl MinecraftWeek {
    fn handle_logistics_input(&mut self, input: &mut input::Input) {
        if input.consume_key_press("escape") {
            input.request_quit = !input.request_quit;
        }
        if input.consume_key_release("keyq") {
            input.request_grab = !input.request_grab;
        }
        if input.consume_key_press("keyr") {
            for (index, pipe) in self.avaliable_pipelines.iter().enumerate() {
                if pipe == &self.pipeline {
                    self.pipeline =
                        self.avaliable_pipelines[(index + 1) % self.avaliable_pipelines.len()].to_owned();
                    break;
                }
            }
        }
        if input.consume_key_press("equal") {
            self.world.view_distance = self.world.view_distance.saturating_add(1);
            self.world.center_chunk = glam::IVec3::MAX;
        }
        if input.consume_key_press("minus") {
            self.world.view_distance = self.world.view_distance.saturating_sub(1);
            self.world.center_chunk = glam::IVec3::MAX;
        }
        if input.consume_key_press("keyg") {
            self.world.chunks.clear();
            self.world.render_chunks.clear();
        }
    }

    fn handle_movement_input(&mut self, input: &mut input::Input) {
        if input.consume_key_press("digit1") {
            self.player.movespeed *= 0.5;
        }
        if input.consume_key_press("digit2") {
            self.player.movespeed *= 2.0;
        }

        let [mut dx, mut dy, mut dz] = [0.0; 3];
        if input.get_key_pres("keyw") {
            dz += 1.0;
        }
        if input.get_key_pres("keys") {
            dz -= 1.0;
        }
        if input.get_key_pres("keyd") {
            dx += 1.0;
        }
        if input.get_key_pres("keya") {
            dx -= 1.0;
        }
        if input.get_key_pres("space") {
            dy += 1.0;
        }
        if input.get_key_pres("shiftleft") {
            dy -= 1.0;
        }
        [dx, dy, dz] = (glam::vec3(dx, dy, dz).normalize_or_zero() * self.player.movespeed).to_array();
        self.camera.update_position(dx, dy, dz);

        let [mut dy, mut dx] = input.consume_mouse_delta().into();
        [dy, dx] = (glam::vec2(dy, dx) * self.player.lookspeed).to_array();
        self.camera.yaw -= dy;
        self.camera.pitch -= dx;
        self.camera.confine_euler();
        self.camera.inner.rotation = glam::Quat::from_rotation_z(0.0)
            * glam::Quat::from_rotation_y(self.camera.yaw)
            * glam::Quat::from_rotation_x(self.camera.pitch);
    }

    fn update_resources(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer) {
        if let Some(resource::GfxResource::Uniform(cam_view_proj)) = render.resources.get("camera_uni") {
            cam_view_proj.write(context, &self.camera.view_proj());
        }
        if let Some(resource::GfxResource::Uniform(cam_view)) = render.resources.get("camera_view_uni") {
            cam_view.write(context, &self.camera.view());
        }
        if let Some(resource::GfxResource::Uniform(global_time)) = render.resources.get("time_uni") {
            global_time.write(context, &self.time);
        }
    }
}
