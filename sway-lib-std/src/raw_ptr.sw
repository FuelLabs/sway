library;

use ::ops::*;

impl PartialEq for raw_ptr {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for raw_ptr {}

impl raw_ptr {
    pub fn null() -> raw_ptr {
        __transmute::<u64, raw_ptr>(0)
    }

    /// Returns `true` if the pointer is null.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the pointer is null, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(2);
    ///     assert(!ptr.is_null());
    /// }
    /// ```
    pub fn is_null(self) -> bool {
        let null_ptr = asm() {
            zero: raw_ptr
        };
        self == null_ptr
    }

    /// Calculates the offset from the pointer.
    ///
    /// # Arguments
    ///
    /// * `count`: [u64] - The number of `size_of<T>` bytes to increase by.
    ///
    /// # Returns
    ///
    /// * [raw_ptr] - The pointer to the offset memory location.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(2);
    ///     let offset_ptr = ptr.add::<u64>(1);
    ///     assert(ptr != offset_ptr);
    /// }
    /// ```
    pub fn add<T>(self, count: u64) -> Self {
        __ptr_add::<T>(self, count)
    }

    /// Calculates the offset from the pointer.
    ///
    /// # Arguments
    ///
    /// * `count`: [u64] - The number of `size_of<T>` bytes to decrease by.
    ///
    /// # Returns
    ///
    /// * [raw_ptr] - The pointer to the offset memory location.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(2);
    ///     let offset_ptr = ptr.add::<u64>(1);
    ///     let subbed_offset = offset_ptr.sub::<u64>(1);
    ///     assert(ptr == subbed_offset);
    /// }
    /// ```
    pub fn sub<T>(self, count: u64) -> Self {
        __ptr_sub::<T>(self, count)
    }

    /// Reads the given type of value from the address.
    ///
    /// # Returns
    ///
    /// * [T] - The copy of the value in memory at the location of the pointer.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     ptr.write(5);
    ///     assert(ptr.read::<u64>() == 5);
    /// }
    /// ```
    pub fn read<T>(self) -> T {
        if __is_reference_type::<T>() {
            asm(ptr: self) {
                ptr: T
            }
        } else if __size_of::<T>() == 1 {
            asm(ptr: self, val) {
                lb val ptr i0;
                val: T
            }
        } else {
            asm(ptr: self, val) {
                lw val ptr i0;
                val: T
            }
        }
    }

    /// Copies `count * size_of<T>` bytes from `self` to `dst`.
    ///
    /// # Arguments
    ///
    /// * `dst`: [raw_ptr] - Pointer to the location in memory to copy the bytes to.
    /// * `count`: [u64] - The number of `size_of<T>` bytes to copy.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr_1 = alloc::<u64>(1);
    ///     let ptr_2 = alloc::<u64>(1);
    ///     ptr_1.write(5);
    ///     ptr_1.copy_to::<u64>(ptr_2, 1);
    ///     assert(ptr_2.read::<u64>() == 5);
    /// }
    /// ```
    pub fn copy_to<T>(self, dst: Self, count: u64) {
        let len = count * __size_of::<T>();
        asm(dst: dst, src: self, len: len) {
            mcp dst src len;
        };
    }

    /// Writes the given value to the address.
    ///
    /// # Arguments
    ///
    /// * `val`: [T] - The value to write to memory.
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     ptr.write(5);
    ///     assert(ptr.read::<u64>() == 5);
    /// }
    /// ```
    pub fn write<T>(self, val: T) {
        if __is_reference_type::<T>() {
            asm(dst: self, src: val, count: __size_of_val(val)) {
                mcp dst src count;
            };
        } else if __size_of::<T>() == 1 {
            asm(ptr: self, val: val) {
                sb ptr val i0;
            };
        } else {
            asm(ptr: self, val: val) {
                sw ptr val i0;
            };
        }
    }

    /// Writes the given byte to the address.
    ///
    /// # Arguments
    ///
    /// * `val`: [u8] - The bytes to write.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u8>(1);
    ///     ptr.write_byte(5u8);
    ///     assert(ptr.read::<u8>() == 5u8);
    /// }
    /// ```
    pub fn write_byte(self, val: u8) {
        let val_ptr = asm(r1: val) {
            r1: raw_ptr
        };
        asm(ptr: self, val: val_ptr) {
            sb ptr val i0;
        };
    }

    /// Reads a byte from the given address.
    ///
    /// # Returns
    ///
    /// * [u8] - The byte in memory.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u8>(1);
    ///     ptr.write_byte(5u8);
    ///     assert(ptr.read_byte() == 5u8);
    /// }
    /// ```
    pub fn read_byte(self) -> u8 {
        asm(r1: self, r2) {
            lb r2 r1 i0;
            r2: u8
        }
    }

    /// Copies `count` bytes from `self` to `dst`.
    ///
    /// # Arguments
    ///
    /// * `dst`: [raw_ptr] - Pointer to the location in memory to copy the bytes to.
    /// * `count`: [u64] - The number of bytes to copy.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr_1 = alloc::<u8>(1);
    ///     let ptr_2 = alloc::<u8>(1);
    ///     ptr_1.write_byte(5u8);
    ///     ptr_1.copy_bytes_to(ptr_2, 1);
    ///     assert(ptr_2.read_byte() == 5u8);
    /// }
    /// ```
    pub fn copy_bytes_to(self, dst: Self, count: u64) {
        asm(dst: dst, src: self, len: count) {
            mcp dst src len;
        };
    }

    /// Add a `u64` offset to a `raw_ptr`.
    ///
    /// # Arguments
    ///
    /// * `count`: [u64] - The number of `u64` bytes to increase by.
    ///
    /// # Returns
    ///
    /// * [raw_ptr] - The pointer to the offset memory location.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(2);
    ///     let offset_ptr_1 = ptr.add::<u64>(1);
    ///     let offset_ptr_2 = ptr.add_uint_offset(1);
    ///     assert(offset_ptr_1 == offset_ptr_2);
    /// }
    /// ```
    pub fn add_uint_offset(self, offset: u64) -> Self {
        asm(ptr: self, offset: offset, new) {
            add new ptr offset;
            new: raw_ptr
        }
    }

    /// Subtract a `u64` offset from a `raw_ptr`.
    ///
    /// # Arguments
    ///
    /// * `count`: [u64] - The number of `u64` bytes to decrease by.
    ///
    /// # Returns
    ///
    /// * [raw_ptr] - The pointer to the offset memory location.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::alloc::alloc;
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(2);
    ///     let offset_ptr = ptr.add::<u64>(1);
    ///     let subbed_offset = offset_ptr.sub_uint_offset(1);
    ///     assert(ptr == subbed_offset);
    /// }
    /// ```
    pub fn sub_uint_offset(self, offset: u64) -> Self {
        asm(ptr: self, offset: offset, new) {
            sub new ptr offset;
            new: raw_ptr
        }
    }
}
