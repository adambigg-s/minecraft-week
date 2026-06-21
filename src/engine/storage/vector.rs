use std::ops;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vector<T, const N: usize>
{
     items: [T; N],
}

impl<T, const N: usize> Vector<T, N>
where
     T: Copy,
{
     #[inline(always)]
     pub fn to_array(self) -> [T; N]
     {
          self.items
     }

     #[inline(always)]
     pub fn len(&self) -> usize
     {
          N
     }

     #[inline(always)]
     pub fn is_empty(&self) -> bool
     {
          N == 0
     }

     #[inline(always)]
     pub fn iter(&self) -> std::slice::Iter<'_, T>
     {
          self.items.iter()
     }

     #[inline(always)]
     pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T>
     {
          self.items.iter_mut()
     }
}

impl<const N: usize> Vector<f64, N>
{
     #[inline(always)]
     pub fn norm(&self) -> f64
     {
          let mut accum = 0.0;
          (0 .. N).for_each(|i| accum += self.items[i] * self.items[i]);
          accum.sqrt()
     }
}

impl<T, const N: usize> Default for Vector<T, N>
where
     T: Default + Copy,
{
     #[inline(always)]
     fn default() -> Self
     {
          Self {
               items: [T::default(); N],
          }
     }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, N>
{
     #[inline(always)]
     fn from(items: [T; N]) -> Self
     {
          Self {
               items,
          }
     }
}

impl<T, const N: usize> From<Vector<T, N>> for [T; N]
{
     #[inline(always)]
     fn from(value: Vector<T, N>) -> Self
     {
          value.items
     }
}

impl<T, const N: usize> ops::Add for Vector<T, N>
where
     T: Copy + ops::Add<Output = T>,
{
     type Output = Self;

     #[inline(always)]
     fn add(mut self, rhs: Self) -> Self::Output
     {
          (0 .. N).for_each(|i| {
               self.items[i] = self.items[i] + rhs.items[i];
          });
          self
     }
}

impl<T, const N: usize> ops::Sub for Vector<T, N>
where
     T: Clone + Copy + ops::Sub<Output = T>,
{
     type Output = Self;

     #[inline(always)]
     fn sub(mut self, rhs: Self) -> Self::Output
     {
          (0 .. N).for_each(|i| {
               self.items[i] = self.items[i] - rhs.items[i];
          });
          self
     }
}

impl<T, D, const N: usize> ops::Mul<D> for Vector<T, N>
where
     T: Copy + ops::Mul<D, Output = T>,
     D: Copy,
{
     type Output = Self;

     #[inline(always)]
     fn mul(mut self, rhs: D) -> Self::Output
     {
          (0 .. N).for_each(|i| {
               self.items[i] = self.items[i] * rhs;
          });
          self
     }
}

impl<T, D, const N: usize> ops::Div<D> for Vector<T, N>
where
     T: Copy + ops::Div<D, Output = T>,
     D: Copy,
{
     type Output = Self;

     #[inline(always)]
     fn div(mut self, rhs: D) -> Self::Output
     {
          (0 .. N).for_each(|i| {
               self.items[i] = self.items[i] / rhs;
          });
          self
     }
}

impl<T, const N: usize> ops::Deref for Vector<T, N>
{
     type Target = [T; N];

     #[inline(always)]
     fn deref(&self) -> &Self::Target
     {
          &self.items
     }
}

impl<T, const N: usize> ops::DerefMut for Vector<T, N>
{
     #[inline(always)]
     fn deref_mut(&mut self) -> &mut Self::Target
     {
          &mut self.items
     }
}
