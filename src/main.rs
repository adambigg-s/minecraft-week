pub mod application;
pub mod atlas;
pub mod block;
pub mod chunk;
pub mod engine;
pub mod mesher;
pub mod minecraft_week;
pub mod pipelines;
pub mod player;
pub mod render;
pub mod skybox;
pub mod terrain;

fn main() -> anyhow::Result<()> {
    application::run::<minecraft_week::MinecraftWeek>()?;
    Ok(())
}
