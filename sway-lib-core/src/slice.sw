library;

use ::raw_ptr::*;

/// Returns a `__slice[T]` from a pointer and length.
///
/// # Arguments
///
/// * `parts`: [(raw_ptr, u64)] - A location in memory and a length to become a `__slice[T]`.
///
/// # Returns
///
/// * [__slice[T]] - The newly created `__slice[T]`.
fn from_parts<T>(parts: (raw_ptr, u64)) -> __slice[T] {
    asm(ptr: parts) {
        ptr: __slice[T]
    }
}

/// Returns a pointer and length from a `__slice[T]`.
///
/// # Arguments
///
/// * `slice`: [__slice[T]] - The slice to be broken into its parts.
///
/// # Returns
///
/// * [(raw_ptr, u64)] - A tuple of the location in memory of the original `__slice[T]` and its length.
fn into_parts<T>(slice: __slice[T]) -> (raw_ptr, u64) {
    asm(ptr: slice) {
        ptr: (raw_ptr, u64)
    }
}

impl<T> __slice[T] {
    /// Forms a slice from a pointer and a length.
    ///
    /// # Arguments
    ///
    /// * `ptr`: [raw_ptr] - The pointer to the location in memory.
    /// * `count`: [u64] - The number of `__size_of::<T>` bytes.
    ///
    /// # Returns
    ///
    /// * [__slice[T]] - The newly created `__slice[T]`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     let slice = slice::from_parts::<u64>(ptr, 1);
    ///     assert(slice.len() == 1);
    /// }
    /// ```
    pub fn from_parts(ptr: raw_ptr, count: u64) -> Self {
        from_parts((ptr, __mul(count, __size_of::<T>())))
    }

    /// Returns the pointer to the slice.
    ///
    /// # Returns
    ///
    /// * [raw_ptr] - The pointer to the location in memory of the `__slice[T]`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     let slice = slice::from_parts::<u64>(ptr, 1);
    ///     let slice_ptr = slice.ptr();
    ///     assert(slice_ptr == ptr);
    /// }
    /// ```
    pub fn ptr(self) -> raw_ptr {
        into_parts(self).0
    }

    /// Returns the number of elements in the slice.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the slice based on `size_of::<T>`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     let slice = raw_slice::from_parts::<u64>(ptr, 1);
    ///     assert(slice.len() == 1);
    /// }
    /// ```
    pub fn len(self) -> u64 {
        __div(into_parts(self).1, __size_of::<T>())
    }
}
