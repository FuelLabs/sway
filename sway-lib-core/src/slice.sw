library;

use ::ptr::*;

pub trait AsSlice<T> {
    fn as_slice(self) -> __slice[T];
}

impl<T> __slice[T] {
    // /// Returns the pointer to the slice.
    // pub fn ptr(self) -> __ptr[T] {
    //     __slice_ptr::<T>(self)
    // }

    /// Returns the number of elements in the slice.
    pub fn len(self) -> u64 {
        __slice_len::<T>(self)
    }
}
