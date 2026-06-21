use std::mem;
use std::ops;

#[derive(Debug)]
pub struct Vec<T, const N: usize>
{
     len: usize,
     items: [mem::MaybeUninit<T>; N],
}

impl<T, const N: usize> Vec<T, N>
{
     const ITEM: mem::MaybeUninit<T> = mem::MaybeUninit::uninit();
     const ITEMS: [mem::MaybeUninit<T>; N] = [Self::ITEM; N];

     pub fn new() -> Self
     {
          Self {
               len: Default::default(),
               items: Self::ITEMS,
          }
     }

     pub fn from_parts(len: usize, items: [T; N]) -> Self
     {
          debug_assert!(len < N);
          Self {
               len,
               items: items.map(|item| mem::MaybeUninit::new(item)),
          }
     }

     pub fn capacity(&self) -> usize
     {
          N
     }

     pub fn len(&self) -> usize
     {
          self.len
     }

     pub fn is_empty(&self) -> bool
     {
          self.len == 0
     }

     pub fn clear(&mut self)
     {
          self.len = 0;
     }

     pub fn push(&mut self, item: T)
     {
          debug_assert!(self.len != N);
          self.items[self.len] = mem::MaybeUninit::new(item);
          self.len += 1;
     }

     pub fn pop(&mut self) -> T
     {
          debug_assert!(self.len != 0);
          self.len -= 1;
          unsafe { self.items[self.len].assume_init_read() }
     }

     pub fn peek(&self) -> &T
     {
          debug_assert!(self.len != 0);
          unsafe { self.items[self.len - 1].assume_init_ref() }
     }
}

impl<T, const N: usize> Default for Vec<T, N>
{
     fn default() -> Self
     {
          Self {
               len: Default::default(),
               items: Self::ITEMS,
          }
     }
}

impl<T, const N: usize> ops::Index<usize> for Vec<T, N>
{
     type Output = T;

     fn index(&self, index: usize) -> &Self::Output
     {
          unsafe { self.items[index].assume_init_ref() }
     }
}

impl<T, const N: usize> ops::IndexMut<usize> for Vec<T, N>
{
     fn index_mut(&mut self, index: usize) -> &mut Self::Output
     {
          unsafe { self.items[index].assume_init_mut() }
     }
}

impl<'d, T, const N: usize> IntoIterator for &'d Vec<T, N>
{
     type Item = &'d T;
     type IntoIter = VecIter<'d, T, N>;

     fn into_iter(self) -> Self::IntoIter
     {
          VecIter {
               inner: self,
               idx: Default::default(),
          }
     }
}

impl<T, const N: usize> FromIterator<T> for Vec<T, N>
{
     fn from_iter<A>(iter: A) -> Self
     where
          A: IntoIterator<Item = T>,
     {
          let mut out = Self::new();
          iter.into_iter().for_each(|item| {
               out.push(item);
          });
          out
     }
}

pub struct VecIter<'d, T, const N: usize>
{
     inner: &'d Vec<T, N>,
     idx: usize,
}

impl<'d, T, const N: usize> Iterator for VecIter<'d, T, N>
{
     type Item = &'d T;

     fn next(&mut self) -> Option<Self::Item>
     {
          if self.idx < self.inner.len()
          {
               let item = unsafe { self.inner.items[self.idx].assume_init_ref() };
               self.idx += 1;
               return Some(item);
          }

          None
     }
}
