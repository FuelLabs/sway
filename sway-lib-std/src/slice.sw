library;

use ::raw_ptr::*;

impl<T> &__slice[T] {
    pub fn ptr(self) -> raw_ptr {
        let (ptr, _) = __transmute::<&__slice[T], (raw_ptr, u64)>(self);
        ptr
    }

    pub fn len(self) -> u64 {
        let (_, len) = __transmute::<&__slice[T], (raw_ptr, u64)>(self);
        len
    }
}
