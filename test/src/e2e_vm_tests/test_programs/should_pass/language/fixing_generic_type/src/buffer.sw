library buffer;

use ::pointer::*;

/// A block of data on the heap
pub struct Buffer {
    ptr: Pointer,
    len: u64,
}

impl Buffer {
    /// Allocates a new buffer with the given size
    pub fn new() -> Self {
        Buffer {
            ptr: alloc(0),
            len: 0,
        }
    }

    /// Allocates a new buffer with the given size
    pub fn alloc(size: u64) -> Self {
        Buffer {
            ptr: alloc(size),
            len: size,
        }
    }

    /// Creates a new buffer from the given value
    pub fn from<T>(val: T) -> Self {
        let ptr = if __is_reference_type::<T>() {
            Pointer::from(val)
        } else {
            Pointer::from((val))
        };
        Buffer {
            ptr: ptr,
            len: __size_of::<T>(),
        }
    }

    /// Pointer to the buffer's data in memory
    pub fn ptr(self) -> Pointer {
        self.ptr
    }

    /// Size of the buffer
    pub fn len(self) -> u64 {
        self.len
    }

    /// Writes the given data into the buffer without doing any checks
    pub fn write_unchecked(self, ptr: Pointer, len: u64, offset: u64) {
        copy(self.ptr.with_offset(offset), ptr, len);
    }

    /// Writes the given value into the buffer
    pub fn write<T>(self, val: T, offset: u64) {
        if __is_reference_type::<T>() {
            let ptr = Pointer::from(val);
            let len = __size_of::<T>();


            // We can't reference `self.write_unchecked` like this:
            // self.write_unchecked(ptr, len, offset);
            // Instead we inline
            copy(self.ptr.with_offset(offset), ptr, len);
        } else {
            let dst_ptr = self.ptr.with_offset(offset);
            asm(ptr: dst_ptr.val, val: val) {
                sw ptr val i0;
            };
        }
    }

    // Non-generic alias to workaround generics bugs
    pub fn write_bool(self, val: bool, offset: u64) {
        let dst_ptr = self.ptr.with_offset(offset);
        asm(ptr: dst_ptr.val, val: val) {
            sw ptr val i0;
        };
    }

    // Non-generic alias to workaround generics bugs
    pub fn write_u64(self, val: u64, offset: u64) {
        let dst_ptr = self.ptr.with_offset(offset);
        asm(ptr: dst_ptr.val, val: val) {
            sw ptr val i0;
        };
    }

    /// Resizes the buffer to the given size
    pub fn resize(self, size: u64) {
        let new_ptr = if self.len == 0 {
            alloc(size)
        } else {
            realloc(self.ptr, self.len, size)
        };


        // We don't have a mut ref to self so we can't do the following:
        // self.ptr = new_ptr;
        // self.len = size;
        // Instead we just copy what we want into self's memory
        asm(r1: self, r2: (new_ptr, size), r3: __size_of::<Buffer>()) {
            mcp r1 r2 r3;
        };
    }

    /// Returns the buffer as the given type without doing any checks
    pub fn into_unchecked<T>(self) -> T {
        self.ptr.into_unchecked()
    }
}

impl core::ops::Eq for Buffer {
    fn eq(self, other: Self) -> bool {
        if self.len == other.len {
            cmp(self.ptr, other.ptr, self.len)
        } else {
            false
        }
    }
}
