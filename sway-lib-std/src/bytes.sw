script;

use std::{alloc::{alloc, realloc}, vec::Vec};
use std::assert::assert;
use std::option::Option;
use std::logging::log;

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
}

impl Bytes {
    pub fn push(ref mut self, item: u8) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.capacity() {
            self.buf.grow();
        };

        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        let end = self.buf.ptr().add::<u8>(self.len);

        // Write `item` at pointer `end`
        end.write(item);

        // Increment length.
        self.len += 1;
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
        log(222);
        let mut bytes = Bytes::new();
        let mut i = 0;
        let length = raw.len();
        assert(raw.len() == 3);
        while i < length {
            log(1212);
            log(i);
            // @review unsure the following unwrap is safe.
            bytes.push(raw.get(i).unwrap());
            bytes.len += 1;
            i += 1;
        };

        bytes
    }
}
fn main() -> bool {
    let mut vec = Vec::new();
    assert(vec.len() == 0);
    vec.push(3u8);
    vec.push(5u8);
    vec.push(7u8);
    assert(vec.len() == 3);
    log(111);
    let bytes = Bytes::from_vec_u8(vec);
    log(999);
    log(bytes.len); // 6 ! expected 3
    assert(bytes.len == 3);
    true
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
fn test_cap() {
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
