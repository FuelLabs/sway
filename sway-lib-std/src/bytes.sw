//! The `Bytes` type is used when a collection of tightly-packed arbitrary bytes is needed.
library;

use ::{alloc::{alloc_bytes, realloc_bytes}, vec::Vec};
use ::assert::{assert, assert_eq};
use ::intrinsics::size_of_val;
use ::option::Option::{self, *};
use ::convert::{From, Into, *};
use ::clone::Clone;
use ::codec::*;
use ::debug::*;
use ::raw_slice::*;
use ::ops::*;
use ::iterator::*;

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

impl From<raw_slice> for RawBytes {
    /// Creates a `RawBytes` from a `raw_slice`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std:bytes::RawBytes;
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
    /// let vec_as_raw_slice = vec.as_raw_slice();
    /// let raw_bytes = RawBytes::from(vec_as_raw_slice);
    ///
    /// assert(raw_bytes.capacity == 3);
    /// ```
    fn from(slice: raw_slice) -> Self {
        let cap = slice.number_of_bytes();
        let ptr = alloc_bytes(cap);
        if cap > 0 {
            slice.ptr().copy_to::<u8>(ptr, cap);
        }
        Self { ptr, cap }
    }
}

/// A type used to represent raw bytes. It has ownership over its buffer.
pub struct Bytes {
    /// A barebones struct for the bytes.
    buf: RawBytes,
    /// The number of bytes being stored.
    len: u64,
}

impl AsRawSlice for Bytes {
    /// Returns a raw slice of all of the elements in the type.
    fn as_raw_slice(self) -> raw_slice {
        asm(ptr: (self.buf.ptr, self.len)) {
            ptr: raw_slice
        }
    }
}

impl Bytes {
    /// Constructs a new, empty `Bytes`.
    ///
    /// # Additional Information
    ///
    /// The struct will not allocate until elements are pushed onto it.
    ///
    /// # Returns
    ///
    /// * [Bytes] - A new, empty `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let bytes = Bytes::new();
    ///     assert(bytes.len() == 0);
    ///     assert(bytes.capacity() == 0);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            buf: RawBytes::new(),
            len: 0,
        }
    }

    /// Constructs a new, empty `Bytes` with the specified capacity.
    ///
    /// # Additional Information
    ///
    /// The `Bytes` will be able to hold exactly `capacity` bytes without
    /// reallocating. If `capacity` is zero, the `Bytes` will not allocate.
    ///
    /// It is important to note that although the returned `Bytes` has the
    /// capacity specified, the type will have a zero length.
    ///
    /// # Arguments
    ///
    /// * `capacity`: [u64] - The capacity with which to initialize the `Bytes`.
    ///
    /// # Returns
    ///
    /// * [Bytes] - A new, empty `Bytes` with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::with_capacity(2);
    ///     // does not allocate
    ///     bytes.push(5);
    ///     // does not re-allocate
    ///     bytes.push(10);
    /// }
    /// ```
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            buf: RawBytes::with_capacity(capacity),
            len: 0,
        }
    }

    /// Appends an element to the back of a `Bytes` collection.
    ///
    /// # Arguments
    ///
    /// * `byte`: [u8] - The element to be pushed onto the `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     let a = 5u8;
    ///     let b = 7u8;
    ///     bytes.push(a);
    ///     bytes.push(b);
    ///     assert(bytes.len() == 2);
    /// }
    /// ```
    pub fn push(ref mut self, byte: u8) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.cap {
            self.buf.grow();
        };

        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        let end = self.buf.ptr.add_uint_offset(self.len);

        // Write `byte` at pointer `end`
        end.write_byte(byte);

        // Increment length.
        self.len += 1;
    }

    /// Removes the last element from a `Bytes` and returns it, or `None` if it
    /// is empty.
    ///
    /// # Returns
    ///
    /// * [Option<u8>] - The last element of the `Bytes`, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///
    ///     let res = bytes.pop();
    ///     assert(res.is_none());
    ///
    ///     bytes.push(5);
    ///     let res = bytes.pop();
    ///     assert(res.unwrap() == 5);
    ///     assert(bytes.is_empty());
    /// }
    /// ```
    pub fn pop(ref mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        };
        // Decrement length.
        self.len -= 1;
        let target = self.buf.ptr.add_uint_offset(self.len);

        Some(target.read_byte())
    }

    /// Returns `Some(byte)` at `index`, or `None` if `index` is out of
    /// bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the element to be returned.
    ///
    /// # Returns
    ///
    /// * [Option<u8>] - The element at the specified index, or `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Byte;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5u8);
    ///     bytes.push(10u8);
    ///     bytes.push(15u8);
    ///     let item = bytes.get(1).unwrap();
    ///     assert(item == 10u8);
    ///     let opt = bytes.get(10);
    ///     assert(opt.is_none()); // index out of bounds
    /// }
    /// ```
    pub fn get(self, index: u64) -> Option<u8> {
        // First check that index is within bounds.
        if self.len <= index {
            return None;
        };

        let item_ptr = self.buf.ptr.add_uint_offset(index);

        Some(item_ptr.read_byte())
    }

    /// Fetches the element stored at `index` without bounds checking.
    fn get_unchecked(self, index: u64) -> u8 {
        self.buf.ptr.add_uint_offset(index).read_byte()
    }

    /// Updates an element at position `index` with a new element `value`.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the element to be set.
    /// * `value`: [u8] - The value of the element to be set.
    ///
    /// # Reverts
    ///
    /// * When `index` is greater than or equal to the length of Bytes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     let a = 5u8;
    ///     let b = 7u8;
    ///     let c = 9u8;
    ///     bytes.push(a);
    ///     bytes.push(b);
    ///     bytes.push(c);
    ///
    ///     let d = 11u8;
    ///
    ///     bytes.set(1, d);
    ///
    ///     assert(bytes.len() == 3);
    ///     assert(bytes.get(0).unwrap() == a);
    ///     assert(bytes.get(1).unwrap() == d);
    ///     assert(bytes.get(2).unwrap() == c);
    /// }
    /// ```
    pub fn set(ref mut self, index: u64, value: u8) {
        assert(index < self.len);

        let index_ptr = self.buf.ptr.add_uint_offset(index);

        index_ptr.write_byte(value);
    }

    /// Inserts an element at position `index` within the Bytes, shifting all
    /// elements after it to the right.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index at which to insert the element.
    /// * `element`: [u8] - The element to be inserted.
    ///
    /// # Reverts
    ///
    /// * When `index > len`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Byte;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     let a = 11u8;
    ///     let b = 11u8;
    ///     let c = 11u8;
    ///     let d = 11u8;
    ///     bytes.push(a);
    ///     bytes.push(b);
    ///     bytes.push(c);
    ///     bytes.insert(1, d);
    ///
    ///     assert(bytes.get(0).unwrap() == a);
    ///     assert(bytes.get(1).unwrap() == d);
    ///     assert(bytes.get(2).unwrap() == b);
    ///     assert(bytes.get(3).unwrap() == c);
    /// }
    /// ```
    pub fn insert(ref mut self, index: u64, element: u8) {
        assert(index <= self.len);

        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.cap {
            self.buf.grow();
        }

        let start = self.buf.ptr;

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
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the element to be removed.
    ///
    /// # Returns
    ///
    /// * [u8] - The element at the specified index.
    ///
    /// # Reverts
    ///
    /// * When `index >= self.len`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Byte;
    ///
    /// fn foo() {
    ///     let mut bytes = Byte::new();
    ///     bytes.push(5);
    ///     bytes.push(10);
    ///     bytes.push(15);
    ///     let item = bytes.remove(1);
    ///     assert(item == 10);
    ///     assert(bytes.get(0).unwrap() == 5);
    ///     assert(bytes.get(1).unwrap() == 15);
    ///     assert(bytes.get(2).is_none());
    /// }
    /// ```
    pub fn remove(ref mut self, index: u64) -> u8 {
        // Panic if index >= length.
        assert(index < self.len);
        let start = self.buf.ptr;

        let item_ptr = start.add_uint_offset(index);
        // Read the value at `index`
        let ret = item_ptr.read_byte();

        // Shift everything down to fill in that spot.
        let mut i = index;
        while i < self.len - 1 {
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
    /// # Arguments
    ///
    /// * `element1_index`: [u64] - The index of the first element.
    /// * `element2_index`: [u64] - The index of the second element.
    ///
    /// # Reverts
    ///
    /// * When `element1_index` or `element2_index` is greater than or equal to the length of `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     let a = 5u8;
    ///     let b = 7u8;
    ///     let c = 9u8;
    ///     bytes.push(a);
    ///     bytes.push(b);
    ///     bytes.push(c);
    ///
    ///     bytes.swap(0, 1);
    ///
    ///     assert(bytes.get(0).unwrap() == b);
    ///     assert(bytes.get(1).unwrap() == a);
    ///     assert(bytes.get(2).unwrap() == c);
    /// }
    /// ```
    pub fn swap(ref mut self, element1_index: u64, element2_index: u64) {
        assert(element1_index < self.len);
        assert(element2_index < self.len);

        if element1_index == element2_index {
            return;
        }

        let start = self.buf.ptr;

        let element1_ptr = start.add_uint_offset(element1_index);
        let element2_ptr = start.add_uint_offset(element2_index);

        let element1_val = element1_ptr.read_byte();
        element2_ptr.copy_bytes_to(element1_ptr, 1);
        element2_ptr.write_byte(element1_val);
    }

    /// Gets the capacity of the allocation.
    ///
    /// # Returns
    ///
    /// * [u64] - The capacity of the allocation.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let bytes = Bytes::with_capacity(5);
    ///     let cap = bytes.capacity();
    ///     assert(cap == 5);
    /// }
    /// ```
    pub fn capacity(self) -> u64 {
        self.buf.capacity()
    }

    /// Gets the length of the `Bytes`.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     assert(bytes.len() == 0);
    ///     bytes.push(5);
    ///     assert(bytes.len() == 1);
    /// }
    /// ```
    pub fn len(self) -> u64 {
        self.len
    }

    /// Clears the `Bytes`, removing all values.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5);
    ///     bytes.clear()
    ///     assert(bytes.is_empty());
    /// }
    /// ```
    pub fn clear(ref mut self) {
        self.buf = RawBytes::new();
        self.len = 0;
    }

    /// Returns `true` if the type contains no elements.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the type contains no elements, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     assert(bytes.is_empty());
    ///     bytes.push(5);
    ///     assert(!bytes.is_empty());
    ///     bytes.clear()
    ///     assert(bytes.is_empty());
    /// }
    /// ```
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Gets the pointer of the allocation.
    ///
    /// # Returns
    ///
    /// [raw_ptr] - The location in memory that the allocated bytes live.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let bytes = Bytes::new();
    ///     assert(!bytes.ptr().is_null());
    /// }
    /// ```
    pub fn ptr(self) -> raw_ptr {
        self.buf.ptr
    }

    /// Divides one Bytes into two at an index.
    ///
    /// # Additional Information
    ///
    /// The first will contain all indices from `[0, mid)` (excluding the index
    /// `mid` itself) and the second will contain all indices from `[mid, len)`
    /// (excluding the index `len` itself).
    ///
    /// # Arguments
    ///
    /// * `mid`: [u64] - Index at which the Bytes is to be split.
    ///
    /// # Reverts
    ///
    /// * When `mid > self.len`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std:bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5u8);
    ///     bytes.push(7u8);
    ///     bytes.push(9u8);
    ///     assert(bytes.len() == 3);
    ///     let mid = 1;
    ///     let (left, right) = bytes.split_at(mid);
    ///     assert(left.capacity() == mid);
    ///     assert(right.capacity() == bytes.len() - mid);
    ///     assert(left.len() == 1);
    ///     assert(right.len() == 2);
    /// }
    /// ```
    pub fn split_at(self, mid: u64) -> (Self, Self) {
        assert(self.len >= mid);

        let left_len = mid;
        let right_len = self.len - mid;

        let mut left_bytes = Self {
            buf: RawBytes::with_capacity(left_len),
            len: left_len,
        };
        let mut right_bytes = Self {
            buf: RawBytes::with_capacity(right_len),
            len: right_len,
        };

        if mid > 0 {
            self.buf.ptr.copy_bytes_to(left_bytes.buf.ptr, left_len);
        };
        if mid != self.len {
            self.buf
                .ptr
                .add_uint_offset(mid)
                .copy_bytes_to(right_bytes.buf.ptr, right_len);
        };

        left_bytes.len = left_len;
        right_bytes.len = right_len;
        (left_bytes, right_bytes)
    }

    /// Appends copies of all elements of `other` into `self`.
    ///
    /// # Additional Information
    ///
    /// NOTE: Appending `self` to itself will duplicate the `Bytes`. i.e. [0, 1, 2] => [0, 1, 2, 0, 1, 2]
    /// This function differs from the Rust `append` function in that it does not clear the `other` `Bytes`.
    ///
    /// # Arguments
    ///
    /// * `other`: [Bytes] - The `Bytes` to append to `self`.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// use std:bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5u8);
    ///     bytes.push(7u8);
    ///     bytes.push(9u8);
    ///     assert(bytes.len() == 3);
    ///
    ///     let mut bytes2 = Bytes::new();
    ///     bytes2.push(5u8);
    ///     bytes2.push(7u8);
    ///     bytes2.push(9u8);
    ///     assert(bytes2.len() == 3);
    ///
    ///     let first_length = bytes.len();
    ///     let second_length = bytes2.len();
    ///
    ///     bytes.append(bytes2);
    ///
    ///     assert(bytes.len() == first_length + second_length);
    ///     assert(bytes2.len() == second_length);
    /// }
    /// ```
    pub fn append(ref mut self, ref mut other: Self) {
        self.append_raw_slice(other.as_raw_slice());
    }

    /// Appends copies of all bytes from the `slice` into `self`.
    ///
    /// # Arguments
    ///
    /// * `slice`: [raw_slice] - The `raw_slice` from which to append to `self`.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// use std:bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5u8);
    ///     bytes.push(7u8);
    ///     bytes.push(9u8);
    ///
    ///     let mut bytes2 = Bytes::new();
    ///     bytes2.push(5u8);
    ///     bytes2.push(7u8);
    ///     bytes2.push(9u8);
    ///
    ///     let first_length = bytes.len();
    ///     let second_length = bytes2.len();
    ///
    ///     bytes.append_raw_slice(bytes2.as_raw_slice());
    ///
    ///     assert(bytes.len() == first_length + second_length);
    /// }
    /// ```
    pub fn append_raw_slice(ref mut self, slice: raw_slice) {
        let slice_len = slice.number_of_bytes();
        if slice_len == 0 {
            return;
        };

        let both_len = self.len + slice_len;
        let other_start = self.len;

        // reallocate with combined capacity, write `slice`, set buffer capacity
        if self.buf.cap < both_len {
            let new_slice = raw_slice::from_parts::<u8>(
                realloc_bytes(self.buf.ptr, self.buf.cap, both_len),
                both_len,
            );
            self.buf = RawBytes::from(new_slice);
        }

        let new_ptr = self.buf.ptr.add_uint_offset(other_start);
        slice.ptr().copy_bytes_to(new_ptr, slice_len);

        // set length
        self.len = both_len;
    }

    /// Removes and returns a range of elements from the `Bytes` (i.e. indices `[start, end)`),
    /// then replaces that range with the contents of `replace_with`.
    ///
    /// # Arguments
    ///
    /// * `start`: [u64] - The starting index for the splice (inclusive).
    /// * `end`: [u64] - The ending index for the splice (exclusive).
    /// * `replace_with`: [Bytes] - The elements to insert in place of the removed range.
    ///
    /// # Returns
    ///
    /// * [Bytes] - A new `Bytes` containing all of the elements from `start` up to (but not including) `end`.
    ///
    /// # Reverts
    ///
    /// * When `start > end`.
    /// * When `end > self.len`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5u8);  // index 0
    ///     bytes.push(7u8);  // index 1
    ///     bytes.push(9u8);  // index 2
    ///
    ///     // Replace the middle item (index 1) with two new items
    ///     let mut replacement = Bytes::new();
    ///     replacement.push(42u8);
    ///     replacement.push(100u8);
    ///
    ///     // Splice out range [1..2) => removes the single element 7u8,
    ///     // then inserts [42, 100] there
    ///     let spliced = bytes.splice(1, 2, replacement);
    ///
    ///     // `spliced` has the element [7u8]
    ///     assert(spliced.len() == 1);
    ///     assert(spliced.get(0).unwrap() == 7u8);
    ///
    ///     // `bytes` is now [5u8, 42u8, 100u8, 9u8]
    ///     assert(bytes.len() == 4);
    ///     assert(bytes.get(0).unwrap() == 5u8);
    ///     assert(bytes.get(1).unwrap() == 42u8);
    ///     assert(bytes.get(2).unwrap() == 100u8);
    ///     assert(bytes.get(3).unwrap() == 9u8);
    /// }
    /// ```
    pub fn splice(ref mut self, start: u64, end: u64, replace_with: Bytes) -> Bytes {
        assert(start <= end);
        assert(end <= self.len);

        let splice_len = end - start;
        let replace_len = replace_with.len;

        // Build the Bytes to return
        let mut spliced = Bytes::with_capacity(splice_len);
        if splice_len > 0 {
            let old_ptr = self.buf.ptr.add_uint_offset(start);
            old_ptr.copy_bytes_to(spliced.buf.ptr, splice_len);
            spliced.len = splice_len;
        }

        // New self
        let new_len = self.len - splice_len + replace_len;
        let mut new_buf = Bytes::with_capacity(new_len);

        // Move head
        if start > 0 {
            let old_ptr = self.buf.ptr;
            old_ptr.copy_bytes_to(new_buf.buf.ptr, start);
        }

        // Move middle
        if replace_len > 0 {
            replace_with
                .buf
                .ptr
                .copy_bytes_to(new_buf.buf.ptr.add_uint_offset(start), replace_len);
        }

        // Move tail
        let tail_len = self.len - end;
        if tail_len > 0 {
            let old_tail = self.buf.ptr.add_uint_offset(end);
            let new_tail = new_buf.buf.ptr.add_uint_offset(start + replace_len);
            old_tail.copy_bytes_to(new_tail, tail_len);
        }

        self.buf = new_buf.buf;
        self.len = new_len;

        spliced
    }

    /// Resizes the `Bytes` in-place so that `len` is equal to `new_len`.
    ///
    /// # Additional Information
    ///
    /// If `new_len` is greater than `len`, the `Bytes` is extended by the difference, with each additional slot filled with `value`. If `new_len` is less than `len`, the `Bytes` is simply truncated.
    ///
    /// # Arguments
    ///
    /// * `new_len`: [u64] - The new length of the `Bytes`.
    /// * `value`: [u8] - The value to fill the new length.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = Bytes::new();
    ///     bytes.resize(1, 7u8);
    ///     assert(bytes.len() == 1);
    ///     assert(bytes.get(0).unwrap() == 7u8);
    ///
    ///     bytes.resize(2, 9u8);
    ///     assert(bytes.len() == 2);
    ///     assert(bytes.get(0).unwrap() == 7u8);
    ///     assert(bytes.get(1).unwrap() == 9u8);
    ///
    ///     bytes.resize(1, 0);
    ///     assert(bytes.len() == 1);
    ///     assert(bytes.get(0).unwrap() == 7u8);
    ///     assert(bytes.get(1) == None);
    /// }
    /// ```
    pub fn resize(ref mut self, new_len: u64, value: u8) {
        // If the `new_len` is less then truncate
        if self.len >= new_len {
            self.len = new_len;
            return;
        }

        // If we don't have enough capacity, alloc more
        if self.buf.cap < new_len {
            self.buf.ptr = realloc_bytes(self.buf.ptr, self.buf.cap, new_len);
            self.buf.cap = new_len;
        }

        // Fill the new length with value
        let mut i = 0;
        let start_ptr = self.buf.ptr.add_uint_offset(self.len);
        while i + self.len < new_len {
            start_ptr.add_uint_offset(i).write_byte(value);
            i += 1;
        }

        self.len = new_len;
    }

    /// Returns an [Iterator] to iterate over this `Bytes`.
    ///
    /// # Returns
    ///
    /// * [BytesIter] - The struct which can be iterated over.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(10_u8);
    ///     bytes.push(15_u8);
    ///
    ///     // Get the iterator
    ///     let iter = bytes.iter();
    ///
    ///     assert_eq(5_u8, iter.next().unwrap());
    ///     assert_eq(10_u8, iter.next().unwrap());
    ///     assert_eq(15_u8, iter.next().unwrap());
    ///
    ///     for elem in bytes.iter() {
    ///         log(elem);
    ///     }
    /// }
    ///
    /// # Undefined Behavior
    ///
    /// Modifying vector during iteration is a logical error and
    /// results in undefined behavior. E.g.:
    ///
    /// ```sway
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(10_u8);
    ///     bytes.push(15_u8);
    ///
    ///     for elem in bytes.iter() {
    ///         bytes.push(20_u8); // Modification causes undefined behavior.
    ///     }
    /// }
    /// ```
    pub fn iter(self) -> BytesIter {
        // WARNING: Be aware of caveats of this implementation
        //          if you take it as an example for implementing
        //          `Iterator` for other types.
        //
        //          Due to the Sway's copy semantics, the `values` will
        //          actually contain **a copy of the original bytes
        //          `self`**. This is contrary to the iterator semantics
        //          which should iterate over the collection itself.
        //
        //          Strictly speaking, we should take a reference to
        //          `self` here, but references as for now an experimental
        //          feature.
        //
        //          However, this issue of copying gets compensated by
        //          another issue, which is the broken copy semantics
        //          for heap types like `Bytes`. Essentially, the original
        //          `self` and it's copy `values` will both point to
        //          the same elements on the heap, which gives us the
        //          desired behavior for the iterator.
        //
        //          This fact makes the implementation of `next` very
        //          misleading in the part where the bytes length is
        //          checked (see comment in the `next` implementation
        //          below).
        //
        //          Once we fix and formalize the copying of heap types
        //          this implementation will be changed, but for
        //          the time being, it is the most pragmatic one we can
        //          have now.
        BytesIter {
            values: self,
            index: 0,
        }
    }

    /// Returns true if all the bytes within the `Bytes` are zero.
    ///
    /// # Additional Information
    ///
    /// If `Bytes` is empty, this function will return `true`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = Bytes::new();
    ///     bytes.resize(10, 0u8);
    ///     assert(bytes.are_all_zero() == true);
    ///
    ///     bytes.resize(20, 42u8);
    ///     assert(bytes.are_all_zero() == false);
    ///
    ///     bytes.resize(0, 42u8);
    ///     assert(bytes.are_all_zero() == true);
    /// }
    /// ```
    pub fn are_all_zero(self) -> bool {
        let mut iter = 0;
        while iter < self.len {
            let item_ptr = self.buf.ptr().add_uint_offset(iter);
            let item = item_ptr.read_byte();
            if item != 0 {
                return false;
            }
            iter += 1;
        }

        true
    }
}

impl PartialEq for Bytes {
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
impl Eq for Bytes {}

// TODO: Once const generics are available implement `From<[u8; N]>`.

/// Methods for converting between the `Bytes` and the `b256` types.
impl From<b256> for Bytes {
    fn from(b: b256) -> Self {
        // Artificially create bytes with capacity and len
        let mut bytes = Self::with_capacity(32);
        bytes.len = 32;
        // Copy bytes from contract_id into the buffer of the target bytes
        __addr_of(b).copy_bytes_to(bytes.buf.ptr, 32);

        bytes
    }
}

impl TryFrom<Bytes> for b256 {
    fn try_from(bytes: Bytes) -> Option<Self> {
        if bytes.len != 32 {
            return None;
        }
        let mut value = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr = __addr_of(value);
        bytes.buf.ptr.copy_to::<b256>(ptr, 1);

        Some(value)
    }
}

impl Into<Bytes> for b256 {
    fn into(self) -> Bytes {
        // Artificially create bytes with capacity and len
        let mut bytes = Bytes::with_capacity(32);
        bytes.len = 32;
        // Copy bytes from contract_id into the buffer of the target bytes
        __addr_of(self).copy_bytes_to(bytes.buf.ptr, 32);

        bytes
    }
}

impl TryInto<b256> for Bytes {
    fn try_into(self) -> Option<b256> {
        if self.len != 32 {
            return None;
        }
        let mut value = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr = __addr_of(value);
        self.buf.ptr.copy_to::<b256>(ptr, 1);

        Some(value)
    }
}

impl From<raw_slice> for Bytes {
    /// Creates a `Bytes` from a `raw_slice`.
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
    /// let vec_as_raw_slice = vec.as_raw_slice();
    /// let bytes = Bytes::from(vec_as_raw_slice);
    ///
    /// assert(bytes.len == 3);
    /// assert(bytes.get(0).unwrap() == a);
    /// assert(bytes.get(1).unwrap() == b);
    /// assert(bytes.get(2).unwrap() == c);
    /// ```
    fn from(slice: raw_slice) -> Self {
        Self {
            buf: RawBytes::from(slice),
            len: slice.number_of_bytes(),
        }
    }
}

impl From<Bytes> for raw_slice {
    /// Creates a `raw_slice` from a `Bytes`.
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
    /// let slice: raw_slice = bytes.into();
    ///
    /// assert(slice.number_of_bytes() == 3);
    /// ```
    fn from(bytes: Bytes) -> raw_slice {
        bytes.as_raw_slice()
    }
}

impl From<Vec<u8>> for Bytes {
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
    /// let bytes = Bytes::from(vec);
    ///
    /// assert(bytes.len == 3);
    /// assert(bytes.get(0).unwrap() == a);
    /// assert(bytes.get(1).unwrap() == b);
    /// assert(bytes.get(2).unwrap() == c);
    /// ```
    fn from(vec: Vec<u8>) -> Self {
        let vec_len = vec.len();
        let mut bytes = Self::with_capacity(vec_len);
        asm(dest: bytes.buf.ptr, src: vec.ptr(), len: vec_len) {
            mcp dest src len;
        }
        bytes.len = vec_len;
        bytes
    }
}

impl From<Bytes> for Vec<u8> {
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
    /// let vec: Vec<u8> = bytes.into();
    ///
    /// assert(vec.len() == 3);
    /// assert(vec.get(0).unwrap() == a);
    /// assert(vec.get(1).unwrap() == b);
    /// assert(vec.get(2).unwrap() == c);
    /// ```
    fn from(bytes: Bytes) -> Vec<u8> {
        bytes.as_raw_slice().into()
    }
}

impl Clone for Bytes {
    fn clone(self) -> Self {
        let len = self.len;
        let buf = RawBytes::with_capacity(len);
        if len > 0 {
            self.buf.ptr.copy_bytes_to(buf.ptr(), len);
        }
        Bytes { buf, len }
    }
}

impl AbiEncode for Bytes {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        self.as_raw_slice().abi_encode(buffer)
    }
}

impl AbiDecode for Bytes {
    fn abi_decode(ref mut buffer: BufferReader) -> Bytes {
        raw_slice::abi_decode(buffer).into()
    }
}

pub struct BytesIter {
    values: Bytes,
    index: u64,
}

impl Iterator for BytesIter {
    type Item = u8;
    fn next(ref mut self) -> Option<Self::Item> {
        // BEWARE: `self.values` keeps **the copy** of the `Bytes`
        //         we iterate over. The below check checks against
        //         the length of that copy, taken when the iterator
        //         was created, and not the original vector.
        //
        //         If the original vector gets modified during the iteration
        //         (e.g., elements are removed), this modification will not
        //         be reflected in `self.values.len`.
        //
        //         But since modifying the vector during iteration is
        //         considered undefined behavior, this implementation,
        //         that always checks against the length at the time
        //         the iterator got created is perfectly valid.
        if self.index >= self.values.len {
            return None
        }

        self.index += 1;
        Some(self.values.get_unchecked(self.index - 1))
    }
}

impl Debug for Bytes {
    fn fmt(self, ref mut f: Formatter) {
        let mut l = f.debug_list();
        for elem in self.iter() {
            let _ = l.entry(elem);
        }
        l.finish();
    }
}
