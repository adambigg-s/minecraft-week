pub mod application;
pub mod engine;
pub mod render;
pub mod minecraft_week;
pub mod pipelines;

fn main() -> anyhow::Result<()> {
    application::run::<minecraft_week::MinecraftWeek>()?;
    Ok(())
}
