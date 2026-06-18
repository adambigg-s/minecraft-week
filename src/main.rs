pub mod application;
pub mod engine;
pub mod minecraft_week;
pub mod render;
pub mod visual;
pub mod world;

fn main() -> anyhow::Result<()> {
    application::run::<minecraft_week::MinecraftWeek>()?;
    Ok(())
}
