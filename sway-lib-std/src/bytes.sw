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
fn ptr_add_offset(start: raw_ptr, offset: u64) -> raw_ptr {
    asm(ptr: start, offset: offset, new) {
        add new ptr offset;
        new: raw_ptr
    }
}

// helper for subtracting a u64 offset from a raw_ptr
fn ptr_sub_offset(start: raw_ptr, offset: u64) -> raw_ptr {
    asm(ptr: start, offset: offset, new) {
        sub new ptr offset;
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
        let target = ptr_add_offset(self.buf.ptr, self.len);

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

        let item_ptr = ptr_add_offset(self.buf.ptr, index);

        Option::Some(item_ptr.read_byte())
    }

    /// Inserts an element at position `index` within the Bytes, shifting all
    /// elements after it to the right.
    ///
    /// ### Reverts
    ///
    /// * If `index > len`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Byte;
    ///
    /// let vec = Vec::new();
    /// let a = 11u8;
    /// let b = 11u8;
    /// let c = 11u8;
    /// let d = 11u8;
    /// vec.push(a);
    /// vec.push(b);
    /// vec.push(c);
    /// bytes.insert(1, d);
    ///
    /// assert(bytes.get(0).unwrap() == a);
    /// assert(bytes.get(1).unwrap() == d);
    /// assert(bytes.get(2).unwrap() == b);
    /// assert(bytes.get(3).unwrap() == c);
    /// ```
    pub fn insert(ref mut self, index: u64, element: u8) {
        assert(index <= self.len);

        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.cap {
            self.buf.grow();
        }

        let start = self.buf.ptr();

        // The spot to put the new value
        let index_ptr = ptr_add_offset(start, index);

        // Shift everything over to make space.
        let mut i = self.len;
        while i > index {
            let idx_ptr = ptr_add_offset(start, i);
            let previous = ptr_sub_offset(idx_ptr, 1);
            previous.copy_bytes_to(idx_ptr, 1);
            i -= 1;
        }

        // Write `element` at pointer `index`
        index_ptr.write_byte(element);

        // Increment length.
        self.len += 1;
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

        let item_ptr = ptr_add_offset(start, index);
        // Read the value at `index`
        let ret = item_ptr.read_byte();

        // Shift everything down to fill in that spot.
        let mut i = index;
        while i < self.len {
            let idx_ptr = ptr_add_offset(start, i);
            let next = ptr_add_offset(idx_ptr, 1);
            next.copy_bytes_to(idx_ptr, 1);
            i += 1;
        }

        // Decrease length.
        self.len -= 1;
        ret
    }

    /// Swaps two elements.
    ///
    /// ### Arguments
    ///
    /// * element1_index - The index of the first element
    /// * element2_index - The index of the second element
    ///
    /// ### Reverts
    ///
    /// * If `element1_index` or `element2_index` is greater than or equal to the length of Bytes.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// let bytes = Bytes::new();
    /// let a = 5u8;
    /// let b = 7u8;
    /// let c = 9u8;
    /// bytes.push(a);
    /// bytes.push(b);
    /// bytes.push(c);
    ///
    /// bytes.swap(0, 1);
    ///
    /// assert(bytes.get(0).unwrap() == b);
    /// assert(bytes.get(1).unwrap() == a);
    /// assert(bytes.get(2).unwrap() == c);
    /// ```
    pub fn swap(ref mut self, element1_index: u64, element2_index: u64) {
        assert(element1_index < self.len);
        assert(element2_index < self.len);

        if element1_index == element2_index {
            return;
        }

        let element1_ptr = ptr_add_offset(self.buf.ptr(), element1_index);
        let element2_ptr = ptr_add_offset(self.buf.ptr(), element2_index);

        let element1_val = element1_ptr.read_byte();
        element2_ptr.copy_bytes_to(element1_ptr, 1);
        element2_ptr.write_byte(element1_val);
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
////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////
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
    let mut bytes = Bytes::new();
    let a = 5u8;
    let b = 7u8;
    let c = 9u8;
    bytes.push(a);
    assert(bytes.len() == 1);
    bytes.push(b);
    assert(bytes.len() == 2);
    bytes.push(c);
    assert(bytes.len() == 3);
}
#[test()]
fn test_pop() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);

    let first = bytes.pop();

    assert(first.unwrap() == c);
    assert(bytes.len() == 2);
}
#[test()]
fn test_len() {
    let (mut bytes, _, _, _) = setup();
    assert(bytes.len() == 3);
}
#[test()]
fn test_clear() {
    let (mut bytes, _, _, _) = setup();
    assert(bytes.len() == 3);

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
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
    assert(bytes.len() == 3);
}

#[test()]
fn test_remove() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);

    let item = bytes.remove(1);

    assert(bytes.len() == 2);
    assert(item == b);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == c);
    assert(bytes.get(2).is_none());
}

#[test()]
fn test_insert() {
    let (mut bytes, a, b, c) = setup();
    let d = 11u8;
    assert(bytes.len() == 3);

    bytes.insert(1, d);

    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == d);
    assert(bytes.get(2).unwrap() == b);
    assert(bytes.get(3).unwrap() == c);
    assert(bytes.len() == 4);
}

#[test()]
fn test_swap() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);

    bytes.swap(0, 1);

    assert(bytes.get(0).unwrap() == b);
    assert(bytes.get(1).unwrap() == a);
    assert(bytes.get(2).unwrap() == c);
    assert(bytes.len() == 3);
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
