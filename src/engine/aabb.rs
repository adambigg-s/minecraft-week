use std::cmp;

use crate::engine::kinematics;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    PosX,
    PosY,
    Posz,
    NegX,
    NegY,
    Negz,
    #[default]
    Undefined,
}

pub struct AaBb<T, const N: usize> {
    pub lo: [T; N],
    pub hi: [T; N],
}

impl<T, const N: usize> AaBb<T, N> {
    pub fn overlaps(&self, other: &Self) -> bool
    where
        T: cmp::PartialOrd,
    {
        (0..N).all(|dim| self.lo[dim] <= other.hi[dim] && self.hi[dim] >= other.lo[dim])
    }
}

impl<T, const N: usize> kinematics::Collision for AaBb<T, N>
where
    T: cmp::PartialOrd,
{
    type Collider = Self;

    fn collides(&self, other: Self::Collider) -> bool {
        self.overlaps(&other)
    }
}
