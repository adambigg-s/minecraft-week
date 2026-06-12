use std::collections;

pub const MOVE_SPEED: f32 = 0.8;
pub const LOOK_SPEED: f32 = 0.0025;
pub const TESTING_GEN: i32 = 1;

use crate::{
    application::{self, input},
    atlas, chunk,
    engine::{camera, transform},
    mesher, pipelines,
    render::{self, GfxCamera, resources, util},
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
    pub chunk_manager: ChunkManager,
    pub pipeline: String,
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
                resources::GfxBindingLayout::Uniform,
                resources::GfxBindingLayout::Texture,
                resources::GfxBindingLayout::Sampler,
            ],
        )?;

        render.register_pipeline::<pipelines::Rainbow>(context, "rainbow_pipe", &["global_layout"]);
        render.register_pipeline::<pipelines::Terrain>(context, "terrain_pipe", &["global_layout"]);
        render.register_pipeline::<pipelines::WireFrame>(context, "wireframe_pipe", &["global_layout"]);

        let atlas = atlas::TextureAtlas::new("./res/", 16)?;
        atlas.save("./res/atlas/texture_atlas.png")?;
        render.register_resource("texture_atlas", util::texture_image(context, &atlas.atlas, "Atlas"));
        render.register_resource("sampler", util::sampler(context, "Sampler"));
        render.register_resource("camera_uni", util::uniform::<glam::Mat4>(context, "Camera"));
        render.register_bind_group(
            context,
            "global_bg",
            "global_layout",
            &["camera_uni", "texture_atlas", "sampler"],
        )?;

        for i in 0..TESTING_GEN {
            for j in 0..TESTING_GEN {
                let random_chunk = chunk::generate_random_chunk(glam::ivec3(i, 0, j));
                render.register_mesh(
                    &format!("chunk_{}x{}_mesh", i, j),
                    mesher::mesh_chunk(context, &atlas, &random_chunk),
                );
            }
        }

        let camera = camera::Camera {
            inner: transform::Transform::from_position([0.0, 0.0, 1.0].into()),
            ar: context.config.width as f32 / context.config.height as f32,
            fov: 67.0,
            znear: 0.1,
            zfear: 500.0,
            ..Default::default()
        };

        let chunk_manager = ChunkManager { chunks: collections::HashMap::new() };

        let pipeline = "terrain_pipe".into();

        Ok(Self { camera, chunk_manager, pipeline })
    }

    fn physics_frame(
        &mut self,
        input: &mut input::Input,
        gfx_context: &render::GfxContext,
        gfx_render: &render::GfxRenderer,
    ) {
        let (context, render) = (gfx_context, gfx_render);

        self.camera.ar = context.config.width as f32 / context.config.height as f32;

        if input.consume_key_press("escape") {
            input.request_quit = !input.request_quit;
        }
        if input.consume_key_release("keyq") {
            input.request_grab = !input.request_grab;
        }
        if input.consume_key_release("keyr") {
            for pipe in render.pipelines.keys() {
                if pipe != &self.pipeline {
                    self.pipeline = pipe.into();
                    break;
                }
            }
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
        [dx, dy, dz] = (glam::vec3(dx, dy, dz).normalize_or_zero() * MOVE_SPEED).to_array();
        self.camera.update_position(dx, dy, dz);

        let [mut dy, mut dx] = input.consume_mouse_delta().into();
        [dy, dx] = (glam::vec2(dy, dx) * LOOK_SPEED).to_array();
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

        if let Some(resources::GfxResource::Uniform(cam)) = render.resources.get("camera_uni") {
            cam.write(context, &self.camera.view_proj());
        }

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
