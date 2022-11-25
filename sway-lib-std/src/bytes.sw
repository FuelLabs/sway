script;

use std::{alloc::{alloc, realloc}, vec::Vec};
use std::assert::assert;
use std::option::Option;
use std::logging::log;
use std::intrinsics::size_of_val;

// HELPERS
// @review word-alignment !?
impl raw_ptr {
    /// Writes the given byte to the address.
    pub fn write_byte(self, val: u8) {
        asm(ptr: self, val: val) {
            sb ptr val i0;
        };
    }
    /// reads a byte from the given address.
    pub fn read_byte(self) -> u8 {
        log(777);
        log(self);
        asm(r1: self, r2) {
            lb r2 r1 i0;
            r2: u8
        }
    }
}

struct RawBytes {
    ptr: raw_ptr,
    cap: u64,
}

impl RawBytes {
    // Create a new `RawBytes` with zero capacity.
    pub fn new() -> Self {
        Self {
            ptr: alloc::<u8>(0),
            cap: 0,
        }
    }

    /// Creates a `RawBytes` (on the heap) with exactly the capacity for a
    /// `[u8; capacity]`. This is equivalent to calling `RawBytes::new` when
    /// `capacity` is `0`.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            ptr: alloc::<u8>(capacity),
            cap: capacity,
        }
    }

    /// Gets the pointer of the allocation.
    pub fn ptr(self) -> raw_ptr {
        self.ptr
    }

    /// Gets the capacity of the allocation.
    pub fn capacity(self) -> u64 {
        self.cap
    }

    /// Grow the capacity of the Bytes by doubling its current capacity. The
    /// `realloc` function allocates memory on the heap and copies the data
    /// from the old allocation to the new allocation
    pub fn grow(ref mut self) {
        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };
        self.ptr = realloc::<u8>(self.ptr, self.cap, new_cap);
        self.cap = new_cap;
    }
}

pub struct Bytes {
    buf: RawBytes,
    len: u64,
}

impl Bytes {
    pub fn new() -> Self {
        Bytes {
            buf: RawBytes::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: u64) -> Self {
        Bytes {
            buf: RawBytes::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn push(ref mut self, item: u8) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.capacity() {
            self.buf.grow();
        };

        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        // let end = self.buf.ptr().add::<u8>(self.len);
        let end = asm(ptr: self.buf.ptr, offset: self.len, new_ptr) {
            add new_ptr ptr offset;
            new_ptr: raw_ptr
        };

        // Write `item` at pointer `end`
        end.write_byte(item);

        // Increment length.
        self.len += 1;
    }

    pub fn pop(ref mut self) -> Option<u8> {
        // log(111);
        if self.len == 0 {
            log(999);
            return Option::None;
        };

        self.len -= 1;
        log(888);
        log(self.len); // 2
        log(self.buf.ptr);// 67108808,
        // let target = self.buf.ptr().add::<u8>(self.len);
        let target = asm(ptr: self.buf.ptr, offset: self.len, new_ptr) {
            add new_ptr ptr offset;
            new_ptr: raw_ptr
        };

        log(target);// 67108824
        // decrement length.
        Option::Some(target.read_byte())
    }

    pub fn capacity(self) -> u64 {
        self.buf.cap
    }

    pub fn len(self) -> u64 {
        self.len
    }

    pub fn clear(ref mut self) {
        self.len = 0;
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }
}

// Need to use seperate impl blocks for now: https://github.com/FuelLabs/sway/issues/1548
impl Bytes {
    // can use From trait when generic traits are in
    pub fn from_vec_u8(ref mut raw: Vec<u8>) -> Self {
        // log(222);
        let mut bytes = Bytes::new();
        let mut i = 0;
        let length = raw.len();
        assert(raw.len() == 3);
        while i < length {
            // log(1212);
            // log(i);
            // @review unsure the following unwrap is safe.
            bytes.push(raw.get(i).unwrap());
            bytes.len += 1;
            i += 1;
        };

        bytes
    }
}
//////////////////////////////////////////////////////////////
fn main() -> bool {
    let mut bytes = Bytes::new();
    log(bytes);   // 0000000004000000_0000000000000000_0000000000000000
    bytes.push(1u8);
    log(bytes); // 0000000003fffff8_0000000000000001_0000000000000001
    //              elements^_____^          cap___^          len___^
    bytes.push(2u8);
    log(bytes);

    bytes.push(3u8);
    log(bytes);

    assert(bytes.len() == 3);

    let first = bytes.pop();
    log(first.unwrap());
    assert(first.unwrap() == 3u8);
    true
}

//////////////////////////////////////////////////////////////
#[test()]
fn test_new_bytes() {
    let bytes = Bytes::new();
    assert(bytes.len() == 0);
}

#[test()]
fn test_push() {
    let mut bytes = Bytes::new();
    bytes.push(5u8);
    bytes.push(7u8);
    bytes.push(9u8);
    assert(bytes.len() == 3);
}

#[test()]
fn test_pop() {
    let mut bytes = Bytes::new();
    bytes.push(5u8);
    bytes.push(7u8);
    bytes.push(9u8);
    assert(bytes.len() == 3);

    let first = bytes.pop();
    assert(first.unwrap() == 9u8);
}

#[test()]
fn test_len() {
    let mut bytes = Bytes::new();
    bytes.push(5u8);
    bytes.push(7u8);
    bytes.push(9u8);
    assert(bytes.len() == 3);
}

#[test()]
fn test_clear() {
    let mut bytes = Bytes::new();
    bytes.push(5u8);
    bytes.push(7u8);
    bytes.push(9u8);
    bytes.clear();
    assert(bytes.len() == 0);
}

#[test()]
fn test_packing() {
    let mut bytes = Bytes::new();
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    assert(bytes.len() == 11);
    assert(bytes.capacity() == 16);
    assert(size_of_val(bytes.buf) == 16);
}

#[test()]
fn test_capacity() {
    let mut bytes = Bytes::new();
    assert(bytes.capacity() == 0);
    bytes.push(5u8);
    assert(bytes.capacity() == 1);
    bytes.push(7u8);
    assert(bytes.capacity() == 2);
    bytes.push(9u8);
    assert(bytes.capacity() == 4);
    bytes.push(11u8);
    assert(bytes.capacity() == 4);
    assert(bytes.len() == 4);
    bytes.push(3u8);
    assert(bytes.capacity() == 8);
    assert(bytes.len() == 5);
}

#[test()]
fn test_from_vec_u8() {
    let mut vec = Vec::new();
    vec.push(11u8);
    vec.push(42u8);
    vec.push(69u8);
    let bytes = Bytes::from_vec_u8(vec);
    assert(bytes.len == 3);
}
