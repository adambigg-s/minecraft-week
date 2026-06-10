use crate::application::input;

pub mod application;
pub mod engine;
pub mod render;

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
        Ok(Self {})
    }

    fn physics_frame(
        &mut self,
        input: &mut input::Input,
        gfx_context: &render::GfxContext,
        gfx_render: &render::GfxRenderer,
    ) {
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
    }
}

fn main() -> anyhow::Result<()> {
    application::run::<MinecraftWeek>()?;
    Ok(())
}
