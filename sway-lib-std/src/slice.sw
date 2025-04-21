library;

use ::raw_ptr::*;

pub fn from_parts<T>(ptr: raw_ptr, count: u64) -> &[T] {
    asm(ptr: (ptr, count)) {
        ptr: &[T]
    }
}

pub fn from_parts_mut<T>(ptr: raw_ptr, count: u64) -> &mut [T] {
    asm(ptr: (ptr, count)) {
        ptr: &mut [T]
    }
}

impl<T> &[T] {
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

    pub fn clone(self) -> &mut [T] {
        let old_ptr = self.ptr();
        let len = self.len();
        let len_in_bytes = __mul(len, __size_of::<T>());

        let new_ptr = asm(len_in_bytes: len_in_bytes, old_ptr: old_ptr) {
            aloc len_in_bytes;
            mcp hp old_ptr len_in_bytes;
            hp: raw_ptr
        };

        asm(buf: (new_ptr, len)) {
            buf: &mut [T]
        }
    }
}

pub fn zero_alloc_slice<T>() -> &mut [T] {
    asm(buf: (0, 0)) {
        buf: &mut [T]
    }
}

pub fn alloc_slice<T>(len: u64) -> &mut [T] {
    let len_in_bytes = __mul(len, __size_of::<T>());
    let ptr = asm(len_in_bytes: len_in_bytes) {
        aloc len_in_bytes;
        hp: raw_ptr
    };
    asm(buf: (ptr, len)) {
        buf: &mut [T]
    }
}

pub fn realloc_slice<T>(old: &mut [T], len: u64) -> &mut [T] {
    let old_ptr = old.ptr();
    let old_len_in_bytes = __mul(old.len(), __size_of::<T>());

    let new_len_in_bytes = __mul(len, __size_of::<T>());
    let new_ptr = asm(
        new_len_in_bytes: new_len_in_bytes,
        old_ptr: old_ptr,
        old_len_in_bytes: old_len_in_bytes,
    ) {
        aloc new_len_in_bytes;
        mcp hp old_ptr old_len_in_bytes;
        hp: raw_ptr
    };

    asm(buf: (new_ptr, len)) {
        buf: &mut [T]
    }
}
