use std::mem;
use std::ops;

#[derive(Debug, Clone)]
pub struct Buffer<T, const N: usize>
{
     size: [usize; N],
     items: Box<[T]>,
}

impl<T, const N: usize> Buffer<T, N>
{
     pub fn new(size: [usize; N]) -> Self
     {
          Self {
               size,
               items: unsafe {
                    mem::transmute::<std::boxed::Box<[std::mem::MaybeUninit<T>]>, std::boxed::Box<[T]>>(
                         Box::<[T]>::new_uninit_slice(size.iter().product()),
                    )
               },
          }
     }

     pub fn from_parts<S>(size: [usize; N], items: S) -> Self
     where
          S: AsRef<[T]> + Into<Box<[T]>>,
     {
          debug_assert!(size.iter().product::<usize>() == items.as_ref().len());
          Self {
               size,
               items: items.into(),
          }
     }

     pub fn size(&self) -> [usize; N]
     {
          self.size
     }

     pub fn fill(&mut self, fill: T)
     where
          T: Copy,
     {
          self.items.iter_mut().for_each(|item| *item = fill);
     }

     pub fn try_get(&self, indices: [usize; N]) -> Option<&T>
     {
          if !self.surrounds(indices)
          {
               return None;
          }
          Some(self.get(indices))
     }

     pub fn get(&self, indices: [usize; N]) -> &T
     {
          let idx = self.linearlize(indices);
          &self.items[idx]
     }

     pub fn try_get_mut(&mut self, indices: [usize; N]) -> Option<&mut T>
     {
          if !self.surrounds(indices)
          {
               return None;
          }
          Some(self.get_mut(indices))
     }

     pub fn get_mut(&mut self, indices: [usize; N]) -> &mut T
     {
          let idx = self.linearlize(indices);
          &mut self.items[idx]
     }

     pub fn linearlize(&self, indices: [usize; N]) -> usize
     {
          debug_assert!(self.surrounds(indices));
          let mut index = 0;
          let mut stride = 1;
          (0 .. N).for_each(|dim| {
               index += indices[dim] * stride;
               stride *= self.size[dim];
          });
          index
     }

     pub fn delinearize(&self, mut index: usize) -> [usize; N]
     {
          debug_assert!(self.size.iter().product::<usize>() > index);
          let mut out = [0; N];
          (0 .. N).rev().for_each(|dim| {
               let modifier = self.size[0 .. dim].iter().product::<usize>();
               out[dim] = index / modifier;
               index -= out[dim] * modifier;
          });
          out
     }

     pub fn surrounds(&self, indices: [usize; N]) -> bool
     {
          (0 .. N).all(|idx| indices[idx] < self.size[idx])
     }
}

impl<T, const N: usize> Buffer<T, N>
where
     T: Default + Clone,
{
     pub fn new_zeroed(size: [usize; N]) -> Self
     {
          Self {
               size,
               items: vec![T::default(); size.iter().product()].into(),
          }
     }
}

impl<T, const N: usize> Default for Buffer<T, N>
{
     fn default() -> Self
     {
          Self {
               size: [0; N],
               items: Default::default(),
          }
     }
}

impl<T, const N: usize> ops::Deref for Buffer<T, N>
{
     type Target = [T];

     fn deref(&self) -> &Self::Target
     {
          &self.items
     }
}

impl<T, const N: usize> ops::DerefMut for Buffer<T, N>
{
     fn deref_mut(&mut self) -> &mut Self::Target
     {
          &mut self.items
     }
}
