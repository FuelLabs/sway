library;

use ::raw_ptr::*;

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
    /// struct MyStruct {
    ///    ptr: raw_ptr,
    ///    len: u64
    /// }
    ///
    /// impl AsRawSlice for MyStruct {
    ///     fn as_raw_slice(self) -> raw_slice {
    ///         from_part(self.ptr, self.len)
    ///     }
    /// }
    /// ```
    fn as_raw_slice(self) -> raw_slice;
}

// Returns a `raw_slice` from a pointer and length.
fn from_parts(parts: (raw_ptr, u64)) -> raw_slice {
    asm(ptr: parts) { ptr: raw_slice }
}

// Returns a pointer and length from a `raw_slice`.
fn into_parts(slice: raw_slice) -> (raw_ptr, u64) {
    asm(ptr: slice) { ptr: (raw_ptr, u64) }
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
    /// }
    /// ```
    pub fn from_parts<T>(ptr: raw_ptr, count: u64) -> Self {
        from_parts((ptr, __mul(count, __size_of::<T>())))
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
        __div(into_parts(self).1, __size_of::<T>())
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
}
