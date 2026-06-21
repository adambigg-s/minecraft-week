use std::cmp;
use std::ops;

use crate::engine::kinematics;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis
{
     PosX,
     PosY,
     Posz,
     NegX,
     NegY,
     Negz,
     #[default]
     Undefined,
}

#[derive(bon::Builder, Debug, Clone, Copy)]
pub struct AaBb<T, const N: usize>
{
     pub lo: [T; N],
     pub hi: [T; N],
}

impl<T, const N: usize> AaBb<T, N>
{
     pub fn new(lo: [T; N], hi: [T; N]) -> Self
     {
          Self {
               lo,
               hi,
          }
     }

     pub fn overlaps(&self, other: &Self) -> bool
     where
          T: cmp::PartialOrd,
     {
          (0 .. N).all(|dim| self.lo[dim] <= other.hi[dim] && self.hi[dim] >= other.lo[dim])
     }

     pub fn point_sides(point: [T; N], sides: [T; N]) -> Self
     where
          T: ops::Add<T, Output = T> + ops::Sub<T, Output = T> + Copy,
     {
          let mut lo = point;
          let mut hi = point;
          (0 .. N).for_each(|dim| {
               lo[dim] = lo[dim] - sides[dim];
               hi[dim] = hi[dim] + sides[dim];
          });
          Self {
               lo,
               hi,
          }
     }
}

impl<T, const N: usize> kinematics::Collision for AaBb<T, N>
where
     T: cmp::PartialOrd,
{
     type Collider = Self;

     fn collides(&self, other: Self::Collider) -> bool
     {
          self.overlaps(&other)
     }
}
