use crate::{
    application::{self, input},
    atlas,
    engine::{camera, transform},
    mesher, pipelines,
    render::{self, GfxCamera, resources, util},
};

#[derive(bon::Builder, Debug)]
pub struct MinecraftWeek {
    camera: camera::Camera,
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

        let atlas = atlas::TextureAtlas::new("./res/", 32)?;
        atlas.save("./res/atlas/texture_atlas.png")?;

        render.register_pipeline::<pipelines::Rainbow>(context, "rainbow_pipe", &["global_layout"]);
        render.register_pipeline::<pipelines::Terrain>(context, "terrain_pipe", &["global_layout"]);

        render.register_mesh(
            "triangle_mesh",
            util::mesh(context, pipelines::TRI_VERTICES, pipelines::TRI_INDICES),
        );
        render.register_resource("camera_uni", util::uniform::<glam::Mat4>(context, "Camera"));
        render.register_resource("sampler", util::sampler(context, "Sampler"));
        render.register_resource(
            "test_texture",
            util::texture(context, "./res/atlas/test_texture.jpg", "Debug grass texture")?,
        );
        render.register_resource("texture_atlas", util::texture_image(context, atlas.atlas, "Texture atlas"));
        render.register_bind_group(
            context,
            "global_bg",
            "global_layout",
            &["camera_uni", "texture_atlas", "sampler"],
        )?;

        render.register_mesh("cube_mesh", mesher::make_cube_mesh(context));

        let camera = camera::Camera {
            inner: transform::Transform::from_position([0.0, 0.0, 1.0].into()),
            ar: context.config.width as f32 / context.config.height as f32,
            fov: 67.0,
            znear: 0.05,
            zfear: 100.0,
        };

        Ok(Self { camera })
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
        [dx, dy, dz] = (glam::vec3(dx, dy, dz).normalize_or_zero() * 0.03).to_array();
        self.camera.update_position(dx, dy, dz);

        let [mut dy, mut dx] = input.consume_mouse_delta().into();
        [dy, dx] = (glam::vec2(dy, dx) * 0.005).to_array();
        self.camera.update_rotation(-dx, -dy, 0.0);
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

        render.queue(render::GfxDrawCall {
            mesh: "triangle_mesh".into(),
            pipe: "rainbow_pipe".into(),
            bind_groups: vec!["global_bg".into()],
        });
        render.queue(render::GfxDrawCall {
            mesh: "cube_mesh".into(),
            pipe: "terrain_pipe".into(),
            bind_groups: vec!["global_bg".into()],
        });
    }
}
