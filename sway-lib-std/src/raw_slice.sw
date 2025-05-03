library;

use ::ops::*;
use ::raw_ptr::*;
use ::slice::*;

/// Trait to return a type as a `raw_slice`.
pub trait AsRawSlice {
    /// Converts self into a `raw_slice`.
    ///
    /// # Returns
    ///
    /// * [raw_slice] - The newly created `raw_slice` from self.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc_bytes;
    ///
    /// struct MyType {
    ///    ptr: raw_ptr,
    ///    len: u64
    /// }
    ///
    /// impl AsRawSlice for MyType {
    ///     fn as_raw_slice(self) -> raw_slice {
    ///         from_parts(self.ptr, self.len)
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let my_type = MyType {
    ///         ptr: alloc_bytes(0),
    ///         len: 0
    ///     }
    ///     let slice = my_type.as_raw_slice();
    ///     assert(slice.ptr() == my_type.ptr);
    ///     assert(slice.number_of_bytes() == my_type.len);
    /// }
    /// ```
    fn as_raw_slice(self) -> raw_slice;
}

/// Returns a `raw_slice` from a pointer and length.
///
/// # Arguments
///
/// * `parts`: [(raw_ptr, u64)] - A location in memory and a length to become a `raw_slice`.
///
/// # Returns
///
/// * [raw_slice] - The newly created `raw_slice`.
fn from_parts(parts: (raw_ptr, u64)) -> raw_slice {
    asm(ptr: parts) {
        ptr: raw_slice
    }
}

/// Returns a pointer and length from a `raw_slice`.
///
/// # Arguments
///
/// * `slice`: [raw_slice] - The slice to be broken into its parts.
///
/// # Returns
///
/// * [(raw_ptr, u64)] - A tuple of the location in memory of the original `raw_slice` and its length.
fn into_parts(slice: raw_slice) -> (raw_ptr, u64) {
    asm(ptr: slice) {
        ptr: (raw_ptr, u64)
    }
}

impl raw_slice {
    /// Forms a slice from a pointer and a length.
    ///
    /// # Arguments
    ///
    /// * `ptr`: [raw_ptr] - The pointer to the location in memory.
    /// * `count`: [u64] - The number of `__size_of::<T>` bytes.
    ///
    /// # Returns
    ///
    /// * [raw_slice] - The newly created `raw_slice`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     let slice = raw_slice::from_parts::<u64>(ptr, 1);
    ///     assert(slice.len::<u64>() == 1);
    /// }
    /// ```
    pub fn from_parts<T>(ptr: raw_ptr, count: u64) -> Self {
        from_parts((ptr, count * __size_of::<T>()))
    }

    /// Returns the pointer to the slice.
    ///
    /// # Returns
    ///
    /// * [raw_ptr] - The pointer to the location in memory of the `raw_slice`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     let slice = raw_slice::from_parts::<u64>(ptr, 1);
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
    ///     assert(slice.len::<u64>() == 1);
    /// }
    /// ```
    pub fn len<T>(self) -> u64 {
        into_parts(self).1 / __size_of::<T>()
    }

    /// Returns the number of elements in the slice when the elements are bytes.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of bytes in the `raw_slice`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     let slice = raw_slice::from_parts::<u64>(ptr, 1);
    ///     assert(slice.number_of_bytes() == 8);
    /// }
    /// ```
    pub fn number_of_bytes(self) -> u64 {
        into_parts(self).1
    }

    pub fn into<T>(self) -> &mut [T] {
        asm(s: into_parts(self)) {
            s: &mut [T]
        }
    }
}

impl<T> AsRawSlice for &mut [T] {
    fn as_raw_slice(self) -> raw_slice {
        from_parts((self.ptr(), self.len()))
    }
}

impl PartialEq for raw_slice {
    fn eq(self, other: Self) -> bool {
        self.ptr() == other.ptr() && self.number_of_bytes() == other.number_of_bytes()
    }
}

impl Eq for raw_slice {}
