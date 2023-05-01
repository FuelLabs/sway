library;


pub fn from_parts<T>(ptr: __ptr[T], len: u64) -> __slice[T] {
    let parts = (ptr, len);
    asm(ptr: parts) { ptr: __slice[T] }
}

pub trait AsSlice<T> {
    fn as_slice(self) -> __slice[T];
}

impl<T> __slice[T] {

    /// Returns the number of elements in the slice.
    pub fn len(self) -> u64 {
        __slice_len::<T>(self)
    }

    /// Returns the pointer to the slice
    pub fn ptr(self) -> __ptr[T] {
        __slice_ptr::<T>(self)
    }
}
