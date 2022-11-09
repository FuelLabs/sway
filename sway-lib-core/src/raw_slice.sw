library raw_slice;

dep raw_ptr;

use ::raw_ptr::*;

pub trait AsRawSlice {
    fn as_raw_slice(self) -> raw_slice;
}

fn from_parts(parts: (raw_ptr, u64)) -> raw_slice {
    asm(ptr: parts) { ptr: raw_slice }
}

fn into_parts(slice: raw_slice) -> (raw_ptr, u64) {
    asm(ptr: slice) { ptr: (raw_ptr, u64) }
}

impl raw_slice {
    /// Forms a slice from a pointer and a length.
    pub fn from_parts<T>(ptr: raw_ptr, count: u64) -> Self {
        from_parts((ptr, __mul(count, __size_of::<T>())))
    }

    /// Returns the pointer to the slice.
    pub fn ptr(self) -> raw_ptr {
        into_parts(self).0
    }

    /// Returns the number of elements in the slice.
    pub fn len<T>(self) -> u64 {
        __div(into_parts(self).1, __size_of::<T>())
    }
}
