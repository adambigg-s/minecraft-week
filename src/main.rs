pub mod application;
pub mod atlas;
pub mod engine;
pub mod minecraft_week;
pub mod pipelines;
pub mod render;
pub mod mesher;

fn main() -> anyhow::Result<()> {
    application::run::<minecraft_week::MinecraftWeek>()?;
    Ok(())
}
