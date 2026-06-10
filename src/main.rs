pub mod application;
pub mod engine;
pub mod render;

pub struct Main {}

impl application::Application for Main {
    fn setup(
        gfx_context: &mut render::GfxContext,
        gfx_render: &mut render::GfxRenderer,
    ) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

fn main() -> anyhow::Result<()> {
    application::run::<Main>()?;
    Ok(())
}
