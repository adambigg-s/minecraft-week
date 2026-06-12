pub mod application;
pub mod atlas;
pub mod terrain;
pub mod engine;
pub mod mesher;
pub mod minecraft_week;
pub mod pipelines;
pub mod render;
pub mod chunk;
pub mod block;

fn main() -> anyhow::Result<()> {
    application::run::<minecraft_week::MinecraftWeek>()?;
    Ok(())
}
