use std::collections;

pub const MOVE_SPEED: f32 = 0.5;
pub const LOOK_SPEED: f32 = 0.0025;
pub const TESTING_GEN: i32 = 12;

use crate::{
    application::{self, input},
    atlas, chunk,
    engine::{camera, transform},
    pipelines,
    render::{self, GfxCamera, resource, util},
    skybox, terrain,
};

#[derive(bon::Builder, Debug)]
pub struct ChunkManager {
    pub chunks: collections::HashMap<glam::IVec2, chunk::Chunk>,
}

impl ChunkManager {
    pub fn generate(&mut self, location: glam::IVec2) {
        if self.chunks.contains_key(&location) {
            return;
        }

        todo!()
    }
}

#[derive(bon::Builder, Debug)]
pub struct MinecraftWeek {
    pub camera: camera::Camera,
    pub movespeed: f32,
    pub lookspeed: f32,
    pub chunk_manager: ChunkManager,
    pub pipeline: String,
    pub avaliable_pipelines: Vec<String>,
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

        render.register_bind_group_layout(
            context,
            "global_layout",
            &[
                resource::GfxBindingLayout::Uniform,
                resource::GfxBindingLayout::Uniform,
                resource::GfxBindingLayout::Texture,
                resource::GfxBindingLayout::Sampler,
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

        render.register_pipeline::<pipelines::Terrain>(context, "terrain_pipe", &["global_layout"]);
        render.register_pipeline::<pipelines::WireFrame>(context, "wireframe_pipe", &["global_layout"]);
        render.register_pipeline::<pipelines::Skybox>(
            context,
            "skybox_pipe",
            &["global_layout", "skybox_layout"],
        );

        let mut skybox = skybox::Skybox::new("./res/skybox/", 32, 500.0)?;
        skybox.texture.save("./res/atlas/skybox_atlas.png")?;
        let atlas = atlas::TextureAtlas::new("./res/", 16)?;
        atlas.save("./res/atlas/texture_atlas.png")?;

        render.register_mesh("skybox_mesh", skybox.create_gfx_mesh(context));

        render.register_resource(
            "skybox_atlas",
            util::texture_image(context, &skybox.texture.atlas, "Skybox atlas"),
        );
        render.register_resource("texture_atlas", util::texture_image(context, &atlas.atlas, "Main atlas"));
        render.register_resource("sampler", util::sampler(context, "Sampler"));
        render.register_resource("camera_uni", util::uniform::<glam::Mat4>(context, "Camera"));
        render.register_resource("camera_view_uni", util::uniform::<glam::Mat4>(context, "Camera view"));

        render.register_bind_group(
            context,
            "global_bg",
            "global_layout",
            &["camera_uni", "camera_view_uni", "texture_atlas", "sampler"],
        )?;
        render.register_bind_group(context, "skybox_bg", "skybox_layout", &["skybox_atlas", "sampler"])?;

        let terrain_gen = terrain::TerrainGenerator::new(1);

        for i in 0..TESTING_GEN {
            for j in 0..TESTING_GEN {
                let time = std::time::Instant::now();
                let chunk = terrain_gen.new_chunk(glam::ivec3(i, 0, j));
                log::warn!(
                    "Chunk generation: {:.3} | {} ms",
                    time.elapsed().as_secs_f32(),
                    time.elapsed().as_millis()
                );
                render.register_mesh(&format!("chunk_{}x{}_mesh", i, j), chunk.mesh(context, &atlas));
                log::warn!(
                    "Chunk meshing: {:.3} | {} ms",
                    time.elapsed().as_secs_f32(),
                    time.elapsed().as_millis()
                );
            }
        }

        let camera = camera::Camera {
            inner: transform::Transform::from_position([0.0, 0.0, 1.0].into()),
            ar: context.config.width as f32 / context.config.height as f32,
            fov: 67.0,
            znear: 0.1,
            zfear: 1000.0,
            ..Default::default()
        };

        let chunk_manager = ChunkManager { chunks: collections::HashMap::new() };

        let pipeline = "terrain_pipe".into();
        let avaliable_pipelines = vec!["terrain_pipe".into(), "wireframe_pipe".into()];

        let lookspeed = LOOK_SPEED;
        let movespeed = MOVE_SPEED;

        Ok(Self {
            camera,
            chunk_manager,
            pipeline,
            avaliable_pipelines,
            lookspeed,
            movespeed,
        })
    }

    fn physics_frame(
        &mut self,
        input: &mut input::Input,
        gfx_context: &render::GfxContext,
        gfx_render: &render::GfxRenderer,
    ) {
        let (context, _) = (gfx_context, gfx_render);

        self.camera.ar = context.config.width as f32 / context.config.height as f32;

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
        if input.consume_key_press("digit1") {
            self.movespeed *= 0.5;
        }
        if input.consume_key_press("digit2") {
            self.movespeed *= 2.0;
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
        [dx, dy, dz] = (glam::vec3(dx, dy, dz).normalize_or_zero() * self.movespeed).to_array();
        self.camera.update_position(dx, dy, dz);

        let [mut dy, mut dx] = input.consume_mouse_delta().into();
        [dy, dx] = (glam::vec2(dy, dx) * self.lookspeed).to_array();
        self.camera.yaw -= dy;
        self.camera.pitch -= dx;
        self.camera.confine_euler();
        self.camera.inner.rotation =
            glam::Quat::from_rotation_y(self.camera.yaw) * glam::Quat::from_rotation_x(self.camera.pitch);
    }

    fn gfx_frame(
        &self,
        _: &input::Input,
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    ) {
        let (context, render) = (gfx_context, gfx_render);

        if let Some(resource::GfxResource::Uniform(cam_view_proj)) = render.resources.get("camera_uni") {
            cam_view_proj.write(context, &self.camera.view_proj());
        }
        if let Some(resource::GfxResource::Uniform(cam_view)) = render.resources.get("camera_view_uni") {
            cam_view.write(context, &self.camera.view());
        }

        render.queue(render::GfxDrawCall {
            mesh: "skybox_mesh".into(),
            pipe: "skybox_pipe".into(),
            bind_groups: vec!["global_bg".into(), "skybox_bg".into()],
        });

        for i in 0..TESTING_GEN {
            for j in 0..TESTING_GEN {
                render.queue(render::GfxDrawCall {
                    mesh: format!("chunk_{}x{}_mesh", i, j),
                    pipe: self.pipeline.to_owned(),
                    bind_groups: vec!["global_bg".into()],
                });
            }
        }
    }
}
