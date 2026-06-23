use std::ops;

use crate::engine::aabb;

pub trait Collision
{
     type Collider;

     fn collides(&self, collider: Self::Collider) -> bool;
}

#[derive(bon::Builder, Debug)]
pub struct Kinematics
{
     pub up: glam::Vec3,
     #[builder(default)]
     pub velocity: glam::Vec3,
     #[builder(default)]
     pub flying: bool,
}

impl Kinematics
{
     const KINEMATIC_DELTA: f32 = 1e-2;

     pub fn jump(&mut self, impulse: f32)
     {
          if self.flying
          {
               return;
          }

          self.velocity += self.up * impulse;
          self.flying = true;
     }

     pub fn apply_gravity(&mut self, impulse: f32, dt: f32)
     {
          self.velocity -= self.up * impulse * dt;
     }

     pub fn check_grounded<Collider>(&mut self, collider: BoxCollider, world: &Collider) -> bool
     where
          Collider: Collision<Collider = BoxCollider>,
     {
          let probe = collider - self.up * Self::KINEMATIC_DELTA;
          world.collides(probe)
     }

     pub fn translate<Collider>(&mut self, start: BoxCollider, world: &Collider, dt: f32) -> BoxCollider
     where
          Collider: Collision<Collider = BoxCollider>,
     {
          let mut curr = start;
          let delta = self.velocity * dt;

          let x = curr + glam::vec3(delta.x, 0.0, 0.0);
          if world.collides(x)
          {
               self.velocity.x = 0.0;
          }
          else
          {
               curr = x;
          }

          let y = curr + glam::vec3(0.0, delta.y, 0.0);
          if world.collides(y)
          {
               self.velocity.y = 0.0;
          }
          else
          {
               curr = y;
          }

          let z = curr + glam::vec3(0.0, 0.0, delta.z);
          if world.collides(z)
          {
               self.velocity.z = 0.0;
          }
          else
          {
               curr = z;
          }

          if self.check_grounded(curr, world) && self.up.dot(self.velocity).abs() < Self::KINEMATIC_DELTA
          {
               self.flying = false;
          }

          curr
     }
}

pub type BoxCollider = aabb::AaBb<f32, 3>;

impl BoxCollider
{
     pub fn center(&self) -> glam::Vec3
     {
          (glam::Vec3::from(self.lo) + glam::Vec3::from(self.hi)) / 2.0
     }
}

impl ops::Add<glam::Vec3> for BoxCollider
{
     type Output = Self;

     fn add(mut self, rhs: glam::Vec3) -> Self::Output
     {
          let rhs = rhs.to_array();
          (0 .. 3).for_each(|dim| {
               self.lo[dim] += rhs[dim];
               self.hi[dim] += rhs[dim];
          });
          self
     }
}

impl ops::Sub<glam::Vec3> for BoxCollider
{
     type Output = Self;

     fn sub(mut self, rhs: glam::Vec3) -> Self::Output
     {
          let rhs = rhs.to_array();
          (0 .. 3).for_each(|dim| {
               self.lo[dim] -= rhs[dim];
               self.hi[dim] -= rhs[dim];
          });
          self
     }
}
