use std::range;

pub trait Cast
{
     type Hit;

     fn cast(&self, ray: Ray) -> Option<Self::Hit>;
}

#[derive(bon::Builder, Debug, Default, Clone, Copy)]
pub struct Ray
{
     pub origin: glam::Vec3,
     pub direction: glam::Vec3,
     pub tspan: range::Range<f32>,
}

impl Ray
{
     pub fn at(&self, time: f32) -> glam::Vec3
     {
          self.origin + self.direction * time
     }
}
