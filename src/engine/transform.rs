use std::ops;

use crate::render;

#[derive(bon::Builder, Debug, Clone, Copy, PartialEq)]
pub struct Transform
{
     #[builder(default)]
     pub scale: glam::Vec3,
     #[builder(default)]
     pub position: glam::Vec3,
     #[builder(default)]
     pub rotation: glam::Quat,
}

impl Transform
{
     pub fn new(position: glam::Vec3, rotation: glam::Quat, scale: glam::Vec3) -> Self
     {
          Self {
               scale,
               position,
               rotation,
          }
     }

     pub fn identity() -> Self
     {
          Self::default()
     }

     pub fn from_position(position: glam::Vec3) -> Self
     {
          Self {
               position,
               ..Default::default()
          }
     }

     pub fn from_rotation(rotation: glam::Quat) -> Self
     {
          Self {
               rotation,
               ..Default::default()
          }
     }

     pub fn from_scale(scale: glam::Vec3) -> Self
     {
          Self {
               scale,
               ..Default::default()
          }
     }

     pub fn to_matrix4(&self) -> glam::Mat4
     {
          glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
     }

     pub fn compose(&self, other: Transform) -> Transform
     {
          Self {
               scale: self.scale * other.scale,
               position: self.position + self.rotation * (self.scale * other.position),
               rotation: self.rotation * other.rotation,
          }
     }

     pub fn inverse(&self) -> Transform
     {
          let inv_rot = self.rotation.inverse();
          let inv_scl = self.scale.recip();
          Self {
               scale: inv_scl,
               rotation: inv_rot,
               position: inv_scl * (inv_rot * -self.position),
          }
     }

     pub fn transform_direction(&self, local_dir: glam::Vec3) -> glam::Vec3
     {
          self.rotation * local_dir
     }

     pub fn forward(&self) -> glam::Vec3
     {
          self.transform_direction(glam::Vec3::NEG_Z)
     }

     pub fn right(&self) -> glam::Vec3
     {
          self.transform_direction(glam::Vec3::X)
     }

     pub fn up(&self) -> glam::Vec3
     {
          self.transform_direction(glam::Vec3::Y)
     }
}

impl Default for Transform
{
     fn default() -> Self
     {
          Self {
               scale: glam::Vec3::ONE,
               position: glam::Vec3::ZERO,
               rotation: glam::Quat::IDENTITY,
          }
     }
}

impl From<glam::Vec3> for Transform
{
     fn from(pos: glam::Vec3) -> Self
     {
          Self::from_position(pos)
     }
}

impl From<glam::Quat> for Transform
{
     fn from(rot: glam::Quat) -> Self
     {
          Self::from_rotation(rot)
     }
}

impl ops::Neg for Transform
{
     type Output = Transform;

     fn neg(self) -> Self::Output
     {
          self.inverse()
     }
}

impl ops::Add for Transform
{
     type Output = Transform;

     fn add(self, rhs: Transform) -> Transform
     {
          self.compose(rhs)
     }
}

impl ops::AddAssign for Transform
{
     fn add_assign(&mut self, rhs: Self)
     {
          *self = *self + rhs
     }
}

impl ops::Sub for Transform
{
     type Output = Transform;

     fn sub(self, rhs: Transform) -> Transform
     {
          self.compose(-rhs)
     }
}

impl ops::SubAssign for Transform
{
     fn sub_assign(&mut self, rhs: Self)
     {
          *self = *self - rhs
     }
}

impl render::GfxTransform for Transform
{
     fn model(&self) -> glam::Mat4
     {
          self.to_matrix4()
     }
}
