pub mod application;
pub mod engine;
pub mod minecraft_week;
pub mod pipelines;
pub mod render;

fn main() -> anyhow::Result<()> {
    application::run::<minecraft_week::MinecraftWeek>()?;
    Ok(())
}
