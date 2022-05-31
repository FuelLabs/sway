//! Library for working with addresses in memory
//! Inspired from: https://doc.rust-lang.org/std/primitive.pointer.html
//! Inspired from: https://doc.rust-lang.org/std/ptr/index.html
library ptr;

use ::assert::*;
use ::intrinsics::*;

/// A point in memory with unknown type
pub struct RawPointer {
    addr: u64,
}

impl RawPointer {
    pub fn new(addr: u64) -> Self {
        RawPointer {
            addr: addr,
        }
    }

    /// Creates a new pointer to the given reference-type value
    pub fn from<T>(val: T) -> Self {
        assert(is_reference_type::<T>());
        RawPointer {
            addr: addr_of(val),
        }
    }

    pub fn addr(self) -> u64 {
        self.addr
    }

    /// Creates a new pointer with the given offset added to the current address
    pub fn add(self, offset: u64) -> Self {
        RawPointer {
            addr: self.addr + offset,
        }
    }

    /// Creates a new pointer with the given offset subtracted from the current address
    pub fn sub(self, offset: u64) -> Self {
        RawPointer {
            addr: self.addr - offset,
        }
    }

    /// Reads the given type of value from the pointer's address
    pub fn read<T>(self) -> T {
        if is_reference_type::<T>() {
            asm(r1: self.addr) {
                r1: T
            }
        } else {
            asm(r1: self.addr) {
                lw r1 r1 i0;
                r1: T
            }
        }
    }

    /// Writes the given value to the pointer's address
    pub fn write<T>(self, val: T) {
        if is_reference_type::<T>() {
            copy(self.addr, addr_of(val), size_of::<T>());
        } else {
            asm(ptr: self.addr, val: val) {
                sw ptr val i0;
            };
        }
    }

    /// Copies the data at the given pointer to the pointer's address
    pub fn copy_from(self, src: Self, len: u64) {
        copy(self.addr, src.addr, len);
    }

    /// Copies the data at the pointer's address to the given pointer
    pub fn copy_to(self, dst: Self, len: u64) {
        copy(dst.addr, self.addr, len);
    }

    // Non-generic aliases to workaround generics bugs
    // See: https://github.com/FuelLabs/sway/issues/1628
    pub fn read_bool(self) -> bool {
        asm(r1: self.addr) {
            lw r1 r1 i0;
            r1: bool
        }
    }
    pub fn read_u64(self) -> u64 {
        asm(r1: self.addr) {
            lw r1 r1 i0;
            r1: u64
        }
    }
    pub fn write_bool(self, val: bool) {
        asm(ptr: self.addr, val: val) {
            sw ptr val i0;
        };
    }
    pub fn write_u64(self, val: u64) {
        asm(ptr: self.addr, val: val) {
            sw ptr val i0;
        };
    }
}

impl core::ops::Eq for RawPointer {
    fn eq(self, other: Self) -> bool {
        self.addr == other.addr
    }
}
