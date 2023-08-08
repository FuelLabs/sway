library;

use ::raw_ptr::*;

impl raw_slice {
    /// Creates a new empty slice without doing any allocation.
    pub fn new() -> Self {
        asm(slice: (0, 0)) {
            slice: Self
        }
    }

    /// Creates a new slice from a pointer and length.
    pub fn from_ptr(ptr: raw_ptr, len: u64) -> Self {
        asm(slice: (ptr, len)) {
            slice: Self
        }
    }

    /// Allocates a new slice with the given length.
    pub fn alloc(len: u64) -> Self {
        asm(len: len, slice: (0, len)) {
            aloc len;
            sw slice hp i0;
            slice: Self
        }
    }

    /// Returns the pointer to the slice.
    pub fn ptr(self) -> raw_ptr {
        asm(slice: self, ptr) {
            lw ptr slice i0;
            ptr: raw_ptr
        }
    }

    /// Returns the number of bytes in the slice.
    pub fn len(self) -> u64 {
        asm(slice: self, len) {
            lw len slice i1;
            len: u64
        }
    }

    /// Reads a single byte from the slice at the given index.
    pub fn read(self, index: u64) -> u8 {
        asm(slice: self, ptr, index: index, val) {
            lw ptr slice i0;
            add ptr ptr index;
            lb val ptr i0;
            val: u8
        }
    }

    /// Writes a single byte to the slice at the given index.
    pub fn write(ref mut self, val: u8, index: u64) {
        asm(slice: self, ptr, index: index, val: val) {
            lw ptr slice i0;
            add ptr ptr index;
            sb ptr val i0;
        }
    }

    /// Returns a new slice pointing to newly allocated memory on the heap.
    pub fn clone(self) -> Self {
        asm(slice: self, ptr, len, new_slice: (0, 0)) {
            lw ptr slice i0;
            lw len slice i1;
            aloc len;
            mcp hp ptr len;
            sw new_slice hp i0;
            sw new_slice len i1;
            new_slice: Self
        }
    }
}

impl raw_slice {
    /// Resizes the slice in-place with the given length.
    /// If the new length is greater than the current length,
    /// the slice is grown by allocating more memory.
    /// If the new length is less than the current length,
    /// the slice is simply truncated.
    /// If the new length is equal to the current length,
    /// nothing happens.
    pub fn resize(ref mut self, new_len: u64) {
        let len = self.len();
        if __gt(new_len, len) {
             if __gt(len, 0) {
                asm(slice: self, ptr, len, new_len: new_len) {
                    lw ptr slice i0;
                    lw len slice i1;
                    aloc new_len;
                    mcp hp ptr len;
                    sw slice hp i0;
                    sw slice new_len i1;
                };
             } else {
                asm(slice: self, new_len: new_len) {
                    aloc new_len;
                    sw slice hp i0;
                    sw slice new_len i1;
                };
             }
        } else if __gt(len, new_len) {
            asm(slice: self, new_len: new_len) {
                sw slice new_len i1;
            };
        }
    }
    
    /// Grows the slice in-place by doubling its current capacity,
    /// or by one if it is empty.
    pub fn grow(ref mut self) {
        let len = self.len();
        if __eq(len, 0) { 
            asm(slice: self, new_len: 1) {
                aloc new_len;
                sw slice hp i0;
                sw slice new_len i1;
            };
        } else {
            asm(slice: self, ptr, len: len, new_len) {
                lw ptr slice i0;
                muli new_len len i2;
                aloc new_len;
                mcp hp ptr len;
                sw slice hp i0;
                sw slice new_len i1;
            };
        }
    }
}

impl raw_slice {
    /// Creates a new slice from a pointer and length.
    pub fn from_ptr_t<T>(ptr: raw_ptr, count: u64) -> Self {
        let len = __mul(count, __size_of::<T>());
        Self::from_ptr(ptr, len)
    }

    /// Allocates a new slice with the given length.
    pub fn alloc_t<T>(count: u64) -> Self {
        let len = __mul(count, __size_of::<T>());
        Self::alloc(len)
    }

    /// Returns the number of elements in the slice.
    pub fn len_t<T>(self) -> u64 {
        __div(self.len(), __size_of::<T>())
    }
}

impl raw_slice {
    /// Reads a single value from the slice at the given index.
    pub fn read_t<T>(self, index: u64) -> T {
        self.ptr().add_t::<T>(index).read_t::<T>()
    }

    /// Writes a single value to the slice at the given index.
    pub fn write_t<T>(ref mut self, val: T, index: u64) {
        self.ptr().add_t::<T>(index).write_t::<T>(val);
    }
    
    /// Reallocates the slice with the given length.
    pub fn resize_t<T>(ref mut self, new_count: u64) {
        let new_len = __mul(new_count, __size_of::<T>());
        self.resize(new_len);
    }
}

impl raw_ptr {
    pub fn into_slice(self, len: u64) -> raw_slice {
        raw_slice::from_ptr(self, len)
    }
}

pub trait AsRawSlice {
    fn as_raw_slice(self) -> raw_slice;
}
