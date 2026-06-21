use std::iter;
use std::ops;

pub fn weighted_sum<V, W, T, D>(values: V, weights: W) -> T
where
     T: Default + ops::Add<T, Output = T> + ops::Mul<D, Output = T>,
     V: IntoIterator<Item = T>,
     W: IntoIterator<Item = D>,
{
     values
          .into_iter()
          .zip(weights)
          .fold(T::default(), |accumulator, (val, weight)| accumulator + val * weight)
}

pub fn weighted_sum_relative<V, W, T, D>(values: V, weights: W) -> T
where
     T: Default + ops::Add<T, Output = T> + ops::Mul<D, Output = T> + ops::Div<D, Output = T>,
     V: IntoIterator<Item = T> + Copy,
     W: IntoIterator<Item = D> + Copy,
     D: iter::Sum + Copy,
{
     let total = weights.into_iter().sum::<D>();
     values
          .into_iter()
          .zip(weights)
          .fold(T::default(), |accumulator, (val, weight)| accumulator + val * (weight) / total)
}
