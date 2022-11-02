library raw_slice;

dep raw_ptr;

use ::raw_ptr::*;

pub trait AsRawSlice {
    fn as_slice(self) -> raw_slice;
}

fn to_parts(slice: raw_slice) -> (raw_ptr, u64) {
    asm(ptr: slice) { ptr: (raw_ptr, u64) }
}

fn from_parts(parts: (raw_ptr, u64)) -> raw_slice {
    asm(ptr: parts) { ptr: raw_slice }
}

impl raw_slice {
    /// Forms a slice from a pointer and a length.
    pub fn from_raw_parts(ptr: raw_ptr, len: u64) -> Self {
        from_parts((ptr, len))
    }

    /// Returns the pointer to the slice.
    pub fn ptr(self) -> raw_ptr {
        to_parts(self).0
    }

    /// Returns the number of elements in the slice.
    pub fn len<T>(self) -> u64 {
        __div(to_parts(self).1, __size_of::<T>())
    }

    /// Calculates the offset from the pointer without modifying the length.
    pub fn add<T>(self, count: u64) -> raw_slice {
        let _self = to_parts(self);
        from_parts((__ptr_add::<T>(_self.0, count), _self.1))
    }

    /// Calculates the offset from the pointer without modifying the length.
    pub fn sub<T>(self, count: u64) -> raw_slice {
        let _self = to_parts(self);
        from_parts((__ptr_sub::<T>(_self.0, count), _self.1))
    }

    /// Copies all bytes from `self` to `dst`.
    /// SAFETY: Length of `dst` will not be checked.
    pub fn copy_to<T>(self, dst: raw_slice) {
        let _self = to_parts(self);
        let _dst = to_parts(dst);
        _self.0.copy_to::<T>(_dst.0, __div(_self.1, __size_of::<T>()));
    }
}
