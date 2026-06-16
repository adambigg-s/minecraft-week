use std::ops;

use crate::engine::aabb;

pub trait Collision {
    type Collider;

    fn collides(&self, collider: Self::Collider) -> bool;
}

pub type BoxCollider = aabb::AaBb<f32, 3>;

impl BoxCollider {
    pub fn center(&self) -> glam::Vec3 {
        (glam::Vec3::from(self.lo) + glam::Vec3::from(self.hi)) / 2.0
    }
}

impl ops::Add<glam::Vec3> for BoxCollider {
    type Output = Self;

    fn add(self, rhs: glam::Vec3) -> Self::Output {
        todo!()
    }
}

impl ops::Sub<glam::Vec3> for BoxCollider {
    type Output = Self;

    fn sub(self, rhs: glam::Vec3) -> Self::Output {
        todo!()
    }
}
