use crate::{
    application::{self, input},
    pipelines,
    render::{self, mesh},
};

#[derive(bon::Builder, Debug)]
pub struct MinecraftWeek {}

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

        render.register_mesh(
            "triangle_mesh",
            mesh::GfxMesh::new(context, pipelines::TRI_VERTICES, pipelines::TRI_INDICES),
        );
        render.register_pipeline::<pipelines::Rainbow>(context, "rainbow_pipe", &[]);

        Ok(Self {})
    }

    fn physics_frame(
        &mut self,
        input: &mut input::Input,
        gfx_context: &render::GfxContext,
        gfx_render: &render::GfxRenderer,
    ) {
        let (context, render) = (gfx_context, gfx_render);

        if input.consume_key_press("escape") {
            input.request_quit = true;
        }
    }

    fn gfx_frame(
        &self,
        input: &input::Input,
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    ) {
        let (context, render) = (gfx_context, gfx_render);

        render.queue(render::GfxDrawCall {
            mesh: "triangle_mesh".into(),
            pipe: "rainbow_pipe".into(),
            bind_groups: vec![],
        });
    }
}
