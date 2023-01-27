//! The `Bytes` type is used when a collection of tightly-packed arbitrary bytes is needed.
library bytes;

use ::{alloc::{alloc_bytes, realloc_bytes}, vec::Vec};
use ::assert::assert;
use ::intrinsics::size_of_val;
use ::option::Option;
use ::convert::From;

struct RawBytes {
    ptr: raw_ptr,
    cap: u64,
}

impl RawBytes {
    /// Create a new `RawBytes` with zero capacity.
    pub fn new() -> Self {
        Self {
            ptr: alloc_bytes(0),
            cap: 0,
        }
    }

    /// Creates a `RawBytes` (on the heap) with exactly the capacity (in bytes) specified.
    /// This is equivalent to calling `RawBytes::new` when `capacity` is zero.
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

    /// Grow the capacity of `Bytes` by doubling its current capacity. The
    /// `realloc_bytes` function allocates memory on the heap and copies
    /// the data from the old allocation to the new allocation.
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
    /// Constructs a new, empty `Bytes`.
    ///
    /// The `Bytes` will not allocate until elements are pushed onto it.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// let bytes = Bytes::new();
    /// // does not allocate
    /// assert(bytes.len() == 0);
    /// assert(bytes.capacity() == 0);
    /// ```
    pub fn new() -> Self {
        Bytes {
            buf: RawBytes::new(),
            len: 0,
        }
    }

    /// Constructs a new, empty `Bytes` with the specified capacity.
    ///
    /// The `Bytes` will be able to hold exactly `capacity` bytes without
    /// reallocating. If `capacity` is zero, the `Bytes` will not allocate.
    ///
    /// It is important to note that although the returned `Bytes` has the
    /// capacity specified, the vector will have a zero length.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// let bytes = Bytes::with_capacity(2);
    /// // does not allocate
    /// bytes.push(5);
    /// // does not re-allocate
    /// bytes.push(10);
    /// ```
    pub fn with_capacity(capacity: u64) -> Self {
        Bytes {
            buf: RawBytes::with_capacity(capacity),
            len: 0,
        }
    }

    /// Appends an element to the back of a `Bytes` collection.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// let mut bytes = Bytes::new();
    /// let a = 5u8;
    /// let b = 7u8;
    /// bytes.push(a);
    /// bytes.push(b);
    /// assert(bytes.len() == 2);
    /// ```
    pub fn push(ref mut self, byte: u8) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.capacity() {
            self.buf.grow();
        };

        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        let end = self.buf.ptr().add_uint_offset(self.len);

        // Write `byte` at pointer `end`
        end.write_byte(byte);

        // Increment length.
        self.len += 1;
    }

    /// Removes the last element from a `Bytes` and returns it, or `None` if it
    /// is empty.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// let bytes = Bytes::new();
    ///
    /// let res = bytes.pop();
    /// assert(res.is_none());
    ///
    /// bytes.push(5);
    /// let res = bytes.pop();
    /// assert(res.unwrap() == 5);
    /// assert(bytes.is_empty());
    /// ```
    pub fn pop(ref mut self) -> Option<u8> {
        if self.len == 0 {
            return Option::None;
        };
        // Decrement length.
        self.len -= 1;
        let target = self.buf.ptr().add_uint_offset(self.len);

        Option::Some(target.read_byte())
    }

    /// Returns `Some(byte)` at `index`, or `None` if `index` is out of
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

        let item_ptr = self.buf.ptr().add_uint_offset(index);

        Option::Some(item_ptr.read_byte())
    }

    /// Updates an element at position `index` with a new element `value`.
    ///
    /// ### Arguments
    ///
    /// * index - The index of the element to be set
    /// * value - The value of the element to be set
    ///
    /// ### Reverts
    ///
    /// * If `index` is greater than or equal to the length of Bytes.
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
    /// let d = 11u8;
    ///
    /// bytes.set(1, d);
    ///
    /// assert(bytes.len() == 3);
    /// assert(bytes.get(0).unwrap() == a);
    /// assert(bytes.get(1).unwrap() == d);
    /// assert(bytes.get(2).unwrap() == c);
    /// ```
    pub fn set(ref mut self, index: u64, value: u8) {
        assert(index < self.len);

        let index_ptr = self.buf.ptr().add_uint_offset(index);

        index_ptr.write_byte(value);
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

        // The spot to put the new value.
        let index_ptr = start.add_uint_offset(index);

        // Shift everything over to make space.
        let mut i = self.len;
        while i > index {
            let idx_ptr = start.add_uint_offset(i);
            let previous = idx_ptr.sub_uint_offset(1);
            previous.copy_bytes_to(idx_ptr, 1);
            i -= 1;
        }

        // Write `element` at pointer `index`.
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
        // Panic if index >= length.
        assert(index < self.len);
        let start = self.buf.ptr();

        let item_ptr = start.add_uint_offset(index);
        // Read the value at `index`
        let ret = item_ptr.read_byte();

        // Shift everything down to fill in that spot.
        let mut i = index;
        while i < self.len {
            let idx_ptr = start.add_uint_offset(i);
            let next = idx_ptr.add_uint_offset(1);
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
    /// * If `element1_index` or `element2_index` is greater than or equal to the length of `Bytes`.
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

        let start = self.buf.ptr();

        let element1_ptr = start.add_uint_offset(element1_index);
        let element2_ptr = start.add_uint_offset(element2_index);

        let element1_val = element1_ptr.read_byte();
        element2_ptr.copy_bytes_to(element1_ptr, 1);
        element2_ptr.write_byte(element1_val);
    }

    /// Gets the capacity of the allocation.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// let bytes = Bytes::with_capacity(5);
    /// let cap = bytes.capacity();
    /// assert(cap == 5);
    /// ```
    pub fn capacity(self) -> u64 {
        self.buf.cap
    }

    pub fn len(self) -> u64 {
        self.len
    }

    /// Clears the `Bytes`, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the Bytes.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// let bytes = Bytes::new();
    /// bytes.push(5);
    /// bytes.clear()
    /// assert(bytes.is_empty());
    /// ```
    pub fn clear(ref mut self) {
        self.buf.ptr = alloc_bytes(0);
        self.len = 0;
        self.buf.cap = 0;
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// let bytes = Bytes::new();
    /// assert(bytes.is_empty());
    /// bytes.push(5);
    /// assert(!bytes.is_empty());
    /// bytes.clear()
    /// assert(bytes.is_empty());
    /// ```
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Returns the `SHA-2-256` hash of the elements.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// let bytes = Bytes::new();
    /// bytes.push(1);
    /// bytes.push(2);
    /// bytes.push(3);
    /// let sha256_hash = bytes.sha256();
    /// ```
    pub fn sha256(self) -> b256 {
        let mut result_buffer = b256::min();
        asm(hash: result_buffer, ptr: self.buf.ptr, bytes: self.len) {
            s256 hash ptr bytes;
            hash: b256
        }
    }

    /// Returns the `KECCAK-256` hash of the elements.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// let bytes = Bytes::new();
    /// bytes.push(1);
    /// bytes.push(2);
    /// bytes.push(3);
    /// let keccak256_hash = bytes.keccak256();
    /// ```
    pub fn keccak256(self) -> b256 {
        let mut result_buffer = b256::min();
        asm(hash: result_buffer, ptr: self.buf.ptr, bytes: self.len) {
            k256 hash ptr bytes;
            hash: b256
        }
    }
}

// Need to use seperate impl blocks for now: https://github.com/FuelLabs/sway/issues/1548
impl Bytes {
    /// Creates a `Bytes` from a `Vec<u8>`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// let mut vec = Vec::new();
    /// let a = 5u8;
    /// let b = 7u8;
    /// let c = 9u8
    ///
    /// vec.push(a);
    /// vec.push(b);
    /// vec.push(c);
    ///
    /// let bytes = Bytes::from_vec_u8(vec);
    ///
    /// assert(bytes.len == 3);
    /// assert(bytes.get(0).unwrap() == a);
    /// assert(bytes.get(1).unwrap() == b);
    /// assert(bytes.get(2).unwrap() == c);
    /// ```
    pub fn from_vec_u8(ref mut vec: Vec<u8>) -> Self {
        let mut bytes = Bytes::new();
        let mut i = 0;
        let length = vec.len();
        while i < length {
            bytes.push(vec.get(i).unwrap());
            i += 1;
        };
        bytes
    }

    /// Creates a `Vec<u8>` from a `Bytes`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// let mut bytes = Bytes::new();
    /// let a = 5u8;
    /// let b = 7u8;
    /// let c = 9u8
    /// bytes.push(a);
    /// bytes.push(b);
    /// bytes.push(c);
    ///
    /// assert(bytes.len() == 3);
    ///
    /// let vec = bytes.into_vec_u8();
    ///
    /// assert(vec.len() == 3);
    /// assert(vec.get(0).unwrap() == a);
    /// assert(vec.get(1).unwrap() == b);
    /// assert(vec.get(2).unwrap() == c);
    /// ```
    pub fn into_vec_u8(self) -> Vec<u8> {
        let mut vec = Vec::new();
        let mut i = 0;
        let length = self.len;
        while i < length {
            vec.push(self.get(i).unwrap());
            i += 1;
        };
        vec
    }

    /// Divides one Bytes into two at an index.
    ///
    /// The first will contain all indices from `[0, mid)` (excluding the index
    /// `mid` itself) and the second will contain all indices from `[mid, len)`
    /// (excluding the index `len` itself).
    ///
    /// ### Arguments
    ///
    /// * mid - Index at which the Bytes is to be split
    ///
    /// ### Reverts
    ///
    /// * if `mid > self.len`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// let (mut bytes, a, b, c) = setup();
    /// assert(bytes.len() == 3);
    /// let mid = 1;
    /// let (left, right) = bytes.split_at(mid);
    /// assert(left.capacity() == mid);
    /// assert(right.capacity() == bytes.len() - mid);
    /// assert(left.len() == 1);
    /// assert(right.len() == 2);
    /// ```
    pub fn split_at(self, mid: u64) -> (Bytes, Bytes) {
        assert(self.len >= mid);

        let left_len = mid;
        let right_len = self.len - mid;

        let mut left_bytes = Self { buf: RawBytes::with_capacity(left_len), len: left_len };
        let mut right_bytes = Self { buf: RawBytes::with_capacity(right_len), len: right_len };

        if mid > 0 {
            self.buf.ptr().copy_bytes_to(left_bytes.buf.ptr(), left_len);
        };
        if mid != self.len {
            self.buf.ptr().add_uint_offset(mid).copy_bytes_to(right_bytes.buf.ptr(), right_len);
        };

        left_bytes.len = left_len;
        right_bytes.len = right_len;

        (left_bytes, right_bytes)
    }

    /// Moves all elements of `other` into `self`, leaving `other` empty.
    ///
    /// ### Arguments
    ///
    /// * other - The Bytes to append to self.
    ///
    /// ### Examples
    ///
    /// ```sway
    ///
    /// use std:bytes::Bytes;
    ///
    ///
    /// let mut bytes = Bytes::new();
    /// bytes.push(5u8);
    /// bytes.push(7u8);
    /// bytes.push(9u8);
    /// assert(bytes.len() == 3);
    ///
    /// let mut bytes2 = Bytes::new();
    /// bytes2.push(5u8);
    /// bytes2.push(7u8);
    /// bytes2.push(9u8);
    /// assert(bytes2.len() == 3);
    ///
    /// let first_length = bytes.len();
    /// let second_length = bytes2.len();
    /// let first_cap = bytes.capacity();
    /// let second_cap = bytes2.capacity();
    /// bytes.append(bytes2);
    /// assert(bytes.len() == first_length + second_length);
    /// assert(bytes.capacity() == first_length + first_length);
    /// ```
    pub fn append(ref mut self, ref other: self) {
        if other.len == 0 {
            return
        };

        // optimization for when starting with empty bytes and appending to it
        if self.len == 0 {
            self = other;
            other.clear();
            return;
        };

        let both_len = self.len + other.len;
        let other_start = self.len;

        // reallocate with combined capacity, write `other`, set buffer capacity
        self.buf.ptr = realloc_bytes(self.buf.ptr(), self.buf.capacity(), both_len);

        let mut i = 0;
        while i < other.len {
            let new_ptr = self.buf.ptr().add_uint_offset(other_start);
            new_ptr.add_uint_offset(i).write_byte(other.buf.ptr.add_uint_offset(i).read_byte());
            i += 1;
        }

        // set capacity and length
        self.buf.cap = both_len;
        self.len = both_len;

        // clear `other`
        other.clear();
    }
}

impl core::ops::Eq for Bytes {
    fn eq(self, other: Self) -> bool {
        if self.len != other.len {
            return false;
        }

        asm(result, r2: self.buf.ptr, r3: other.buf.ptr, r4: self.len) {
            meq result r2 r3 r4;
            result: bool
        }
    }
}

/// Methods for converting between the `Bytes` and the `b256` types.
impl From<b256> for Bytes {
    fn from(b: b256) -> Bytes {
        // Artificially create bytes with capacity and len
        let mut bytes = Bytes::with_capacity(32);
        bytes.len = 32;
        // Copy bytes from contract_id into the buffer of the target bytes
        __addr_of(b).copy_bytes_to(bytes.buf.ptr, 32);

        bytes
    }

    // NOTE: this cas be lossy! Added here as the From trait currently requires it,
    // but the conversion from `Bytes` ->`b256` should be implemented as
    // `impl TryFrom<Bytes> for b256` when the `TryFrom` trait lands:
    // https://github.com/FuelLabs/sway/pull/3881
    fn into(self) -> b256 {
        let mut value = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr = __addr_of(value);
        self.buf.ptr().copy_to::<b256>(ptr, 1);

        value
    }
}

// Tests
//
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
    let (_, a, b, c) = setup();
    let mut bytes = Bytes::new();
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
    bytes.push(42u8);
    bytes.push(11u8);
    bytes.push(69u8);
    bytes.push(100u8);
    bytes.push(200u8);
    bytes.push(255u8);
    bytes.push(180u8);
    bytes.push(17u8);
    bytes.push(19u8);
    assert(bytes.len() == 12);

    let first = bytes.pop();
    assert(first.unwrap() == 19u8);
    assert(bytes.len() == 11);

    let second = bytes.pop();
    assert(second.unwrap() == 17u8);
    assert(bytes.len() == 10);

    let third = bytes.pop();
    assert(third.unwrap() == 180u8);
    assert(bytes.len() == 9);
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    assert(bytes.len() == 3);
    assert(bytes.pop().unwrap() == c);
    assert(bytes.pop().unwrap() == b);
    assert(bytes.pop().unwrap() == a);
    assert(bytes.pop().is_none() == true);
    assert(bytes.len() == 0);
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
    // get is non-modifying
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

    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == b);
    assert(bytes.get(1).unwrap() == a);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn test_set() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    let d = 11u8;

    bytes.set(1, d);

    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == d);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn test_from_vec_u8() {
    let mut vec = Vec::new();
    let (_, a, b, c) = setup();
    vec.push(a);
    vec.push(b);
    vec.push(c);

    let bytes = Bytes::from_vec_u8(vec);

    assert(bytes.len == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn test_into_vec_u8() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);

    let vec = bytes.into_vec_u8();

    assert(vec.len() == 3);
    assert(vec.get(0).unwrap() == a);
    assert(vec.get(1).unwrap() == b);
    assert(vec.get(2).unwrap() == c);
}

#[test()]
fn test_bytes_limits() {
    let mut bytes = Bytes::new();
    let max = 255u8;
    let min = 0u8;
    bytes.push(max);
    bytes.push(min);
    bytes.push(max);
    bytes.push(min);
    bytes.push(max);
    bytes.push(min);

    assert(bytes.len() == 6);
    assert(bytes.capacity() == 8);
    assert(bytes.get(0).unwrap() == max);
    assert(bytes.get(1).unwrap() == min);
    assert(bytes.get(2).unwrap() == max);
    assert(bytes.get(3).unwrap() == min);
    assert(bytes.get(4).unwrap() == max);
    assert(bytes.get(5).unwrap() == min);
}

#[test()]
fn test_split_at() {
    let (mut original, a, b, c) = setup();
    assert(original.len() == 3);
    let index = 1;
    let (left, right) = original.split_at(index);
    assert(original.capacity() == 4);
    assert(right.capacity() == 2);
    assert(left.len() == 1);
    assert(right.len() == 2);
}

#[test()]
fn test_split_at_0() {
    let (mut original, a, b, c) = setup();
    assert(original.len() == 3);
    let index = 0;
    let (left, right) = original.split_at(index);
    assert(original.capacity() == 4);
    assert(right.capacity() == 3);
    assert(left.len() == 0);
    assert(right.len() == 3);
}

#[test()]
fn test_split_at_len() {
    let (mut original, a, b, c) = setup();
    assert(original.len() == 3);
    let index = 3;
    let (left, right) = original.split_at(index);
    assert(original.capacity() == 4);
    assert(right.capacity() == 0);
    assert(left.len() == 3);
    assert(right.len() == 0);
}

#[test()]
fn test_append() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);

    let mut bytes2 = Bytes::new();
    let d = 5u8;
    let e = 7u8;
    let f = 9u8;
    bytes2.push(d);
    bytes2.push(e);
    bytes2.push(f);
    assert(bytes2.len() == 3);
    assert(bytes2.get(0).unwrap() == d);
    assert(bytes2.get(1).unwrap() == e);
    assert(bytes2.get(2).unwrap() == f);

    let first_length = bytes.len();
    let second_length = bytes2.len();
    let first_cap = bytes.capacity();
    let second_cap = bytes2.capacity();
    bytes.append(bytes2);
    assert(bytes.len() == first_length + second_length);
    assert(bytes.capacity() == first_length + first_length);
    let values = [a, b, c, d, e, f];
    let mut i = 0;
    while i < 6 {
        assert(bytes.get(i).unwrap() == values[i]);
        i += 1;
    };
}

#[test()]
fn test_append_empty_bytes() {
    // nothing is appended or modified when appending an empty bytes.
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);

    let mut bytes2 = Bytes::new();
    assert(bytes2.len() == 0);
    let first_length = bytes.len();
    let first_cap = bytes.capacity();
    bytes.append(bytes2);
    assert(bytes.len() == first_length);
    assert(bytes.capacity() == first_cap);
}

#[test()]
fn test_append_to_empty_bytes() {
    let mut bytes = Bytes::new();
    assert(bytes.len() == 0);
    let (mut bytes2, a, b, c) = setup();
    assert(bytes2.len() == 3);

    let first_length = bytes.len();
    let first_cap = bytes.capacity();
    let second_length = bytes2.len();
    let second_cap = bytes2.capacity();
    bytes.append(bytes2);
    assert(bytes.len() == second_length);
    assert(bytes.capacity() == second_cap);
    let values = [a, b, c];
    let mut i = 0;
    while i < 3 {
        assert(bytes.get(i).unwrap() == values[i]);
        i += 1;
    };

    assert(bytes2.len() == 0);
    assert(bytes2.capacity() == 0);

}

#[test()]
fn test_eq() {
    let (mut bytes, a, b, c) = setup();
    let (mut bytes2, a, b, c) = setup();
    assert(bytes == bytes2);

    let d = 5u8;
    let e = 7u8;
    let f = 9u8;
    let mut other = Bytes::new();
    other.push(d);
    other.push(e);
    other.push(f);
    assert(bytes == other);

    other.push(42u8);
    assert(bytes != other);

    bytes.push(42u8);
    assert(bytes == other);

    other.swap(0, 1);
    assert(bytes != other);
}

#[test()]
fn test_sha256() {
    use ::hash::sha256;
    let (mut bytes, a, b, c) = setup();
    bytes.push(0u8);
    bytes.push(0u8);
    bytes.push(0u8);
    bytes.push(0u8);
    bytes.push(0u8);

    // The u8 bytes [5, 7, 9, 0, 0, 0, 0, 0] are equivalent to the u64 integer "362268190631264256"
    assert(sha256(362268190631264256) == bytes.sha256());
}

#[test()]
fn test_keccak256() {
    use ::hash::keccak256;
    let (mut bytes, a, b, c) = setup();
    bytes.push(0u8);
    bytes.push(0u8);
    bytes.push(0u8);
    bytes.push(0u8);
    bytes.push(0u8);

    // The u8 bytes [5, 7, 9, 0, 0, 0, 0, 0] are equivalent to the u64 integer "362268190631264256"
    assert(keccak256(362268190631264256) == bytes.keccak256());
}

#[test]
fn test_from_b256() {
    let initial = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let b: Bytes = Bytes::from(initial);
    let mut control_bytes = Bytes::with_capacity(32);

    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        control_bytes.push(51u8);
        i += 1;
    }

    assert(b == control_bytes);
}

#[test]
fn test_into_b256() {
    let mut initial_bytes = Bytes::with_capacity(32);

    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        initial_bytes.push(51u8);
        i += 1;
    }

    let value: b256 = initial_bytes.into();
    let expected: b256 = 0x3333333333333333333333333333333333333333333333333333333333333333;

    assert(value == expected);
}
