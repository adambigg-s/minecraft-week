pub mod buffer;
pub mod stack;
pub mod vector;

#[inline(always)]
#[allow(clippy::missing_safety_doc)]
pub unsafe fn bit_interp<T, D>(value: &T) -> D
where
     T: Copy,
     D: Copy,
{
     debug_assert!(std::mem::size_of::<D>() != 0 && std::mem::size_of::<T>() != 0);
     unsafe { *(value as *const T as *const D) }
}

#[inline(always)]
#[allow(clippy::mut_from_ref)]
#[allow(invalid_reference_casting)]
#[allow(clippy::missing_safety_doc)]
pub unsafe fn mut_cast<T>(value: &T) -> &mut T
{
     let inter = value as *const T as *mut T;
     unsafe { &mut *inter }
}
