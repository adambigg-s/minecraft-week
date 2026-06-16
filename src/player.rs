use crate::engine::aabb;

#[derive(bon::Builder, Debug)]
pub struct PlayerController {
    pub movespeed: f32,
    pub lookspeed: f32,
    pub collider: aabb::AaBb<f32, 3>,
    #[builder(default)]
    pub collisions: bool,
}
