library bytes;

use ::{alloc::{alloc, realloc}, vec::Vec};
use ::assert::assert;
use ::option::Option;
use ::logging::log;
use ::intrinsics::size_of_val;

impl raw_ptr {
    /// Writes the given byte to the address.
    pub fn write_byte(self, val: u8) {
        let val_ptr = asm(r1: val) { r1: raw_ptr };
        asm(ptr: self, val: val_ptr) {
            sb ptr val i0;
        };
    }
    /// reads a byte from the given address.
    pub fn read_byte(self) -> u8 {
        asm(r1: self, r2) {
            lb r2 r1 i0;
            r2: u8
        }
    }

    pub fn copy_bytes_to(self, dst: raw_ptr, count: u64) {
        asm(dst: dst, src: self, len: count) {
            mcp dst src len;
        };
    }
}

// HELPERS
pub fn alloc_bytes(count: u64) -> raw_ptr {
    asm(size: count, ptr) {
        aloc size;
        addi ptr hp i1;
        ptr: raw_ptr
    }
}

pub fn realloc_bytes(ptr: raw_ptr, count: u64, new_count: u64) -> raw_ptr {
    if new_count > count {
        let new_ptr = alloc_bytes(new_count);
        if count > 0 {
            ptr.copy_bytes_to(new_ptr, count);
        };
        new_ptr
    } else {
        ptr
    }
}

// helper for adding a u64 offset to a raw_ptr
fn ptr_with_offset(start: raw_ptr, offset: u64) -> raw_ptr {
    asm(ptr: start, offset: offset, new) {
        add new ptr offset;
        new: raw_ptr
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
            ptr: alloc_bytes(0),
            cap: 0,
        }
    }

    /// Creates a `RawBytes` (on the heap) with exactly the capacity for a
    /// `[u8; capacity]`. This is equivalent to calling `RawBytes::new` when
    /// `capacity` is `0`.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            ptr: alloc_bytes(capacity),
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
        self.ptr = realloc_bytes(self.ptr, self.cap, new_cap);
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

    pub fn push(ref mut self, byte: u8) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.capacity() {
            self.buf.grow();
        };

        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        let end = asm(ptr: self.buf.ptr, offset: self.len, new_ptr) {
            add new_ptr ptr offset;
            new_ptr: raw_ptr
        };
        // Write `item` at pointer `end`
        end.write_byte(byte);

        // Increment length.
        self.len += 1;
    }

    pub fn pop(ref mut self) -> Option<u8> {
        if self.len == 0 {
            return Option::None;
        };
        // decrement length.
        self.len -= 1;
        let target = ptr_with_offset(self.buf.ptr, self.len);

        Option::Some(target.read_byte())
    }

    /// Returns a byte at `index`, or None if `index` is out of
    /// bounds.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Byte;
    ///
    /// let bytes = Bytes::new();
    /// bytes.push(5u8);
    /// bytes.push(10u8);
    /// bytes.push(15u8);
    /// let item = bytes.get(1).unwrap();
    /// assert(item == 10u8);
    /// let opt = bytes.get(10);
    /// assert(opt.is_none()); // index out of bounds
    /// ```
    pub fn get(self, index: u64) -> Option<u8> {
        // First check that index is within bounds.
        if self.len <= index {
            return Option::None;
        };

        let item_ptr = ptr_with_offset(self.buf.ptr, index);

        Option::Some(item_ptr.read_byte())
    }

    /// Removes and returns the element at position `index` within the Bytes,
    /// shifting all elements after it to the left.
    ///
    /// ### Reverts
    ///
    /// * If `index >= self.len`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Byte;
    ///
    /// let bytes = Byte::new();
    /// bytes.push(5);
    /// bytes.push(10);
    /// bytes.push(15);
    /// let item = bytes.remove(1);
    /// assert(item == 10);
    /// assert(bytes.get(0).unwrap() == 5);
    /// assert(bytes.get(1).unwrap() == 15);
    /// assert(bytes.get(2).is_none());
    /// ```
    pub fn remove(ref mut self, index: u64) -> u8 {
        // panic if index >= length
        assert(index < self.len);
        let start = self.buf.ptr();

        // Read the value at `index`
        let item_ptr = ptr_with_offset(start, index);
        let ret = item_ptr.read_byte();

        // Shift everything down to fill in that spot.
        let mut i = index;
        while i < self.len {
            let idx_ptr = ptr_with_offset(start, i);
            let next = ptr_with_offset(idx_ptr, 1);
            next.copy_bytes_to(idx_ptr, 1);
            i += 1;
        }

        // Decrease length.
        self.len -= 1;
        ret
    }

    // pub fn
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
// impl Bytes {
//     // can use From trait when generic traits are in
//     pub fn from_vec_u8(ref mut raw: Vec<u8>) -> Self {
//         let mut bytes = Bytes::new();
//         let mut i = 0;
//         let length = raw.len();
//         assert(raw.len() == 3);
//         while i < length {
//             // @review unsure the following unwrap is safe.
//             bytes.push(raw.get(i).unwrap());
//             bytes.len += 1;
//             i += 1;
//         };
//         bytes
//     }
// }
//////////////////////////////////////////////////////////////
// fn main() -> bool {
//     let mut bytes = Bytes::new();
//     bytes.push(1u8);
//     // bytes.push(2u8);
//     // bytes.push(3u8);
//     // assert(bytes.len() == 3);
//     let val = bytes.pop().unwrap(); // 111
//     log(val);
//     assert(val == 1u8);
//     true
// }
//////////////////////////////////////////////////////////////
fn setup() -> (Bytes, u8, u8, u8) {
    let mut bytes = Bytes::new();
    let a = 5u8;
    let b = 7u8;
    let c = 9u8;
    bytes.push(a);
    bytes.push(b);
    bytes.push(c);
    (bytes, a, b, c)
}

#[test()]
fn test_new_bytes() {
    let bytes = Bytes::new();
    assert(bytes.len() == 0);
}
#[test()]
fn test_push() {
    let (mut bytes, _, _, _) = setup();
    assert(bytes.len() == 3);
}
#[test()]
fn test_pop() {
    let (mut bytes, _, _, _) = setup();
    assert(bytes.len() == 3);
    let first = bytes.pop();
    assert(first.unwrap() == 9u8);
}
#[test()]
fn test_len() {
    let (mut bytes, _, _, _) = setup();
    assert(bytes.len() == 3);
}
#[test()]
fn test_clear() {
    let (mut bytes, _, _, _) = setup();
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
fn test_get() {
    let (bytes, a, b, c) = setup();
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
    assert(bytes.len() == 3);
}

#[test()]
fn test_remove() {
    let (mut bytes, a, b, c) = setup();
    let item = bytes.remove(1);
    assert(item == b);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == c);
    assert(bytes.get(2).is_none());
    assert(bytes.len() == 2);
}
// #[test()]
// fn test_from_vec_u8() {
//     let mut vec = Vec::new();
//     vec.push(11u8);
//     vec.push(42u8);
//     vec.push(69u8);
//     let bytes = Bytes::from_vec_u8(vec);
//     assert(bytes.len == 3);
// }
