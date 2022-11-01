library pointer;

use std::revert::*;

/// A point in memory with unknown type
pub struct Pointer {
    val: u64,
}

impl Pointer {
    pub fn new(val: u64) -> Self {
        Pointer { val: val }
    }

    /// Creates a new pointer to the given reference-type value
    pub fn from<T>(val: T) -> Self {
        if !__is_reference_type::<T>() {
            revert(0);
        };
        Pointer {
            val: asm(r1: val) { r1: u64 },
        }
    }

    pub fn val(self) -> u64 {
        self.val
    }

    /// Returns a new pointer adjusted by the given offset
    pub fn with_offset(self, offset: u64) -> Self {
        Pointer {
            val: self.val + offset,
        }
    }

    /// Returns the pointee as the given type without doing any checks
    pub fn into_unchecked<T>(self) -> T {
        if __is_reference_type::<T>() {
            asm(r1: self.val) { r1: T }
        } else {
            asm(r1: self.val, r2) {
                lw r2 r1 i0;
                r2: T
            }
        }
    }
}

impl core::ops::Eq for Pointer {
    fn eq(self, other: Self) -> bool {
        self.val == other.val
    }
}

/// Allocates an amount of memory on the heap
pub fn alloc(size: u64) -> Pointer {
    Pointer::new(asm(size: size, ptr) {
        aloc size;
        addi ptr hp i1;
        ptr: u64
    })
}

/// Copies data from source to destination
pub fn copy(dst: Pointer, src: Pointer, size: u64) {
    asm(dst: dst.val(), src: src.val(), size: size) {
        mcp dst src size;
    };
}

/// Compares data at two points of memory
pub fn cmp(first: Pointer, second: Pointer, len: u64) -> bool {
    asm(r1, first: first.val(), second: second.val(), len: len) {
        meq r1 first second len;
        r1: bool
    }
}

/// Reallocates the given area of memory
pub fn realloc(ptr: Pointer, old_size: u64, new_size: u64) -> Pointer {
    if new_size < old_size {
        ptr
    } else {
        let new_ptr = alloc(new_size);
        copy(new_ptr, ptr, old_size);
        new_ptr
    }
}
