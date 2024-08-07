library;

use ::raw_ptr::*;

impl<T> &__slice[T] {
    pub fn ptr(self) -> raw_ptr {
        let (ptr, _) = asm(s: self) {
            s: (raw_ptr, u64)
        };
        ptr
    }

    pub fn len(self) -> u64 {
        let (_, len) = asm(s: self) {
            s: (raw_ptr, u64)
        };
        len
    }
}
