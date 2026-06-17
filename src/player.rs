use crate::engine::{aabb, kinematics};

#[derive(bon::Builder, Debug)]
pub struct PlayerController {
    pub movespeed: f32,
    pub lookspeed: f32,
    pub collider: aabb::AaBb<f32, 3>,
    pub kinematics: kinematics::Kinematics,
    #[builder(default)]
    pub collisions: bool,
}
