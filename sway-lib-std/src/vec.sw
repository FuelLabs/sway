//! A vector type for dynamically sized arrays outside of storage.
library;

use ::alloc::{alloc, realloc};
use ::assert::assert;
use ::option::Option::{self, *};
use ::convert::From;
use ::iterator::*;
use ::codec::*;
use ::debug::*;
use ::ops::*;
use ::raw_slice::*;
use ::clone::Clone;
use ::debug::{Debug, DebugList, Formatter};

struct RawVec<T> {
    ptr: raw_ptr,
    cap: u64,
}

impl<T> RawVec<T> {
    /// Create a new `RawVec` with zero capacity.
    ///
    /// # Returns
    ///
    /// * [RawVec] - A new `RawVec` with zero capacity.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::RawVec;
    ///
    /// fn foo() {
    ///     let vec = RawVec::new();
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            ptr: alloc::<T>(0),
            cap: 0,
        }
    }

    /// Creates a `RawVec` (on the heap) with exactly the capacity for a
    /// `[T; capacity]`. This is equivalent to calling `RawVec::new` when
    /// `capacity` is zero.
    ///
    /// # Arguments
    ///
    /// * `capacity`: [u64] - The capacity of the `RawVec`.
    ///
    /// # Returns
    ///
    /// * [RawVec] - A new `RawVec` with zero capacity.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::RawVec;
    ///
    /// fn foo() {
    ///     let vec = RawVec::with_capacity(5);
    /// }
    /// ```
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            ptr: alloc::<T>(capacity),
            cap: capacity,
        }
    }

    /// Gets the pointer of the allocation.
    ///
    /// # Returns
    ///
    /// * [raw_ptr] - The pointer of the allocation.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::RawVec;
    ///
    /// fn foo() {
    ///     let vec = RawVec::new();
    ///     let ptr = vec.ptr();
    ///     let end = ptr.add::<u64>(0);
    ///     end.write(5);
    ///     assert(end.read::<u64>() == 5);
    /// }
    pub fn ptr(self) -> raw_ptr {
        self.ptr
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
    /// use std::vec::RawVec;
    ///
    /// fn foo() {
    ///     let vec = RawVec::with_capacity(5);
    ///     let cap = vec.capacity();
    ///     assert(cap == 5);
    /// }
    pub fn capacity(self) -> u64 {
        self.cap
    }

    /// Grow the capacity of the vector by doubling its current capacity. The
    /// `realloc` function allocates memory on the heap and copies the data
    /// from the old allocation to the new allocation.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::RawVec;
    ///
    /// fn foo() {
    ///     let mut vec = RawVec::new();
    ///     vec.grow();
    ///     assert(vec.capacity() == 1);
    ///     vec.grow();
    ///     assert(vec.capacity() == 2);
    /// }
    pub fn grow(ref mut self) {
        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };

        self.ptr = realloc::<T>(self.ptr, self.cap, new_cap);
        self.cap = new_cap;
    }
}

impl<T> From<raw_slice> for RawVec<T> {
    fn from(slice: raw_slice) -> Self {
        let cap = slice.len::<T>();
        let ptr = alloc::<T>(cap);
        if cap > 0 {
            slice.ptr().copy_to::<T>(ptr, cap);
        }
        Self { ptr, cap }
    }
}

/// A contiguous growable array type, written as `Vec<T>`, short for 'vector'. It has ownership over its buffer.
pub struct Vec<T> {
    buf: RawVec<T>,
    len: u64,
}

impl<T> Vec<T> {
    /// Constructs a new, empty `Vec<T>`.
    ///
    /// # Additional Information
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// # Returns
    ///
    /// * [Vec] - A new, empty `Vec<T>`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     // allocates when an element is pushed
    ///     vec.push(5);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            buf: RawVec::new(),
            len: 0,
        }
    }

    /// Constructs a new, empty `Vec<T>` with the specified capacity.
    ///
    /// # Additional Information
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is zero, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// *capacity* specified, the vector will have a zero *length*.
    ///
    /// # Arguments
    ///
    /// * `capacity`: [u64] - The capacity of the `Vec<T>`.
    ///
    /// # Returns
    ///
    /// * [Vec<T>] - A new, empty `Vec<T>` with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::with_capacity(2);
    ///     // does not allocate
    ///     vec.push(5);
    ///     // does not re-allocate
    ///     vec.push(10);
    ///     // allocates
    ///     vec.push(15);
    /// }
    /// ```
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            buf: RawVec::with_capacity(capacity),
            len: 0,
        }
    }

    /// Appends an element at the end of the collection.
    ///
    /// # Arguments
    ///
    /// * `value`: [T] - The value to be pushed onto the end of the collection.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     let last_element = vec.pop().unwrap();
    ///     assert(last_element == 5);
    /// }
    ///```
    pub fn push(ref mut self, value: T) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.cap {
            self.buf.grow();
        };

        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        let end = self.buf.ptr.add::<T>(self.len);

        // Write `value` at pointer `end`
        end.write::<T>(value);

        // Increment length.
        self.len += 1;
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
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let vec = Vec::with_capacity(5);
    ///     let cap = vec.capacity();
    ///     assert(cap == 5);
    /// }
    /// ```
    pub fn capacity(self) -> u64 {
        self.buf.cap
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the vector.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.clear()
    ///     assert(vec.is_empty());
    /// }
    /// ```
    pub fn clear(ref mut self) {
        self.len = 0;
    }

    /// Fetches the element stored at `index`
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the element to be fetched.
    ///
    /// # Returns
    ///
    /// * [Option<T>] - The element stored at `index`, or `None` if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///     vec.push(15);
    ///     let item = vec.get(1).unwrap();
    ///     assert(item == 10);
    ///     let res = vec.get(10);
    ///     assert(res.is_none()); // index out of bounds
    /// }
    /// ```
    pub fn get(self, index: u64) -> Option<T> {
        // First check that index is within bounds.
        if self.len <= index {
            return None;
        };

        // Get a pointer to the desired element using `index`
        let ptr = self.buf.ptr.add::<T>(index);

        // Read from `ptr`
        Some(ptr.read::<T>())
    }

    /// Fetches the element stored at `index` without bounds checking.
    fn get_unchecked(self, index: u64) -> T {
        self.buf.ptr.add::<T>(index).read::<T>()
    }

    /// Returns the number of elements in the vector, also referred to
    /// as its `length`.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the vector.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     assert(vec.len() == 1);
    ///     vec.push(10);
    ///     assert(vec.len() == 2);
    /// }
    /// ```
    pub fn len(self) -> u64 {
        self.len
    }

    /// Returns whether the vector is empty.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the vector is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     assert(vec.is_empty());
    ///     vec.push(5);
    ///     assert(!vec.is_empty());
    /// }
    /// ```
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the element to be removed.
    ///
    /// # Returns
    ///
    /// * [T] - The element that was removed.
    ///
    /// # Reverts
    ///
    /// * If `index >= self.len`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///     vec.push(15);
    ///     let item = vec.remove(1);
    ///     assert(item == 10);
    ///     assert(vec.get(0).unwrap() == 5);
    ///     assert(vec.get(1).unwrap() == 15);
    ///     assert(vec.get(2).is_none());
    /// }
    /// ```
    pub fn remove(ref mut self, index: u64) -> T {
        assert(index < self.len);

        let buf_start = self.buf.ptr;

        // Read the value at `index`
        let ptr = buf_start.add::<T>(index);
        let ret = ptr.read::<T>();

        // Shift everything down to fill in that spot.
        let mut i = index;
        if self.len > 1 {
            while i < self.len - 1 {
                let ptr = buf_start.add::<T>(i);
                ptr.add::<T>(1).copy_to::<T>(ptr, 1);
                i += 1;
            }
        }

        // Decrease length.
        self.len -= 1;
        ret
    }

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index at which to insert the element.
    ///
    /// * `element`: [T] - The element to be inserted.
    ///
    /// # Reverts
    ///
    /// * If `index > self.len`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///
    ///     vec.insert(1, 15);
    ///
    ///     assert(vec.get(0).unwrap() == 5);
    ///     assert(vec.get(1).unwrap() == 15);
    ///     assert(vec.get(2).unwrap() == 10);
    /// }
    /// ```
    pub fn insert(ref mut self, index: u64, element: T) {
        assert(index <= self.len);

        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.cap {
            self.buf.grow();
        }

        let buf_start = self.buf.ptr;

        // The spot to put the new value
        let index_ptr = buf_start.add::<T>(index);

        // Shift everything over to make space.
        let mut i = self.len;
        while i > index {
            let ptr = buf_start.add::<T>(i);
            ptr.sub::<T>(1).copy_to::<T>(ptr, 1);
            i -= 1;
        }

        // Write `element` at pointer `index`
        index_ptr.write::<T>(element);

        // Increment length.
        self.len += 1;
    }

    /// Removes the last element from a vector and returns it.
    ///
    /// # Returns
    ///
    /// * [Option<T>] - The last element of the vector, or `None` if the vector is empty.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///
    ///     let res = vec.pop();
    ///     assert(res.is_none());
    ///
    ///     vec.push(5);
    ///     let res = vec.pop();
    ///     assert(res.unwrap() == 5);
    ///     assert(vec.is_empty());
    /// }
    /// ```
    pub fn pop(ref mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(self.buf.ptr.add::<T>(self.len).read::<T>())
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
    /// * If `element1_index` or `element2_index` is greater than or equal to the length of vector.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///
    ///     vec.swap(0, 1);
    ///
    ///     assert(vec.get(0).unwrap() == 10);
    ///     assert(vec.get(1).unwrap() == 5);
    /// }
    /// ```
    pub fn swap(ref mut self, element1_index: u64, element2_index: u64) {
        assert(element1_index < self.len);
        assert(element2_index < self.len);

        if element1_index == element2_index {
            return;
        }

        let element1_ptr = self.buf.ptr.add::<T>(element1_index);
        let element2_ptr = self.buf.ptr.add::<T>(element2_index);

        let element1_val: T = element1_ptr.read::<T>();
        element2_ptr.copy_to::<T>(element1_ptr, 1);
        element2_ptr.write::<T>(element1_val);
    }

    /// Updates an element at position `index` with a new element `value`.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the element to be set.
    /// * `value`: [T] - The value of the element to be set.
    ///
    /// # Reverts
    ///
    /// * If `index` is greater than or equal to the length of vector.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///
    ///     vec.set(0, 15);
    ///
    ///     assert(vec.get(0).unwrap() == 15);
    ///     assert(vec.get(1).unwrap() == 10);
    /// }
    /// ```
    pub fn set(ref mut self, index: u64, value: T) {
        assert(index < self.len);

        let index_ptr = self.buf.ptr.add::<T>(index);

        index_ptr.write::<T>(value);
    }

    /// Returns an [Iterator] to iterate over this `Vec`.
    ///
    /// # Returns
    ///
    /// * [VecIter<V>] - The struct which can be iterated over.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///     vec.push(15);
    ///
    ///     // Get the iterator
    ///     let iter = vec.iter();
    ///
    ///     assert_eq(5, iter.next().unwrap());
    ///     assert_eq(10, iter.next().unwrap());
    ///     assert_eq(15, iter.next().unwrap());
    ///
    ///     for elem in vec.iter() {
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
    ///     let mut vec = Vec::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///     vec.push(15);
    ///
    ///     for elem in vec.iter() {
    ///         vec.push(20); // Modification causes undefined behavior.
    ///     }
    /// }
    /// ```
    pub fn iter(self) -> VecIter<T> {
        // WARNING: Be aware of caveats of this implementation
        //          if you take it as an example for implementing
        //          `Iterator` for other types.
        //
        //          Due to the Sway's copy semantics, the `values` will
        //          actually contain **a copy of the original vector
        //          `self`**. This is contrary to the iterator semantics
        //          which should iterate over the collection itself.
        //
        //          Strictly speaking, we should take a reference to
        //          `self` here, but references as for now an experimental
        //          feature.
        //
        //          However, this issue of copying gets compensated by
        //          another issue, which is the broken copy semantics
        //          for heap types like `Vec`. Essentially, the original
        //          `self` and it's copy `values` will both point to
        //          the same elements on the heap, which gives us the
        //          desired behavior for the iterator.
        //
        //          This fact makes the implementation of `next` very
        //          misleading in the part where the vector length is
        //          checked (see comment in the `next` implementation
        //          below).
        //
        //          Once we fix and formalize the copying of heap types
        //          this implementation will be changed, but for
        //          the time being, it is the most pragmatic one we can
        //          have now.
        VecIter {
            values: self,
            index: 0,
        }
    }

    /// Gets the pointer of the allocation.
    ///
    /// # Returns
    ///
    /// [raw_ptr] - The location in memory that the allocated vec lives.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let vec = Vec::new();
    ///     assert(!vec.ptr().is_null());
    /// }
    /// ```
    pub fn ptr(self) -> raw_ptr {
        self.buf.ptr
    }

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// # Additional Information
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the difference, with each additional slot filled with `value`. If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// # Arguments
    ///
    /// * `new_len`: [u64] - The new length of the `Vec`.
    /// * `value`: [T] - The value to fill the new length.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let vec: Vec<u64> = Vec::new();
    ///     vec.resize(1, 7);
    ///     assert(vec.len() == 1);
    ///     assert(vec.get(0).unwrap() == 7);
    ///
    ///     vec.resize(2, 9);
    ///     assert(vec.len() == 2);
    ///     assert(vec.get(0).unwrap() == 7);
    ///     assert(vec.get(1).unwrap() == 9);
    ///
    ///     vec.resize(1, 0);
    ///     assert(vec.len() == 1);
    ///     assert(vec.get(0).unwrap() == 7);
    ///     assert(vec.get(1) == None);
    /// }
    /// ```
    pub fn resize(ref mut self, new_len: u64, value: T) {
        // If the `new_len` is less then truncate
        if self.len >= new_len {
            self.len = new_len;
            return;
        }

        // If we don't have enough capacity, alloc more
        if self.buf.cap < new_len {
            self.buf.ptr = realloc::<T>(self.buf.ptr, self.buf.cap, new_len);
            self.buf.cap = new_len;
        }

        // Fill the new length with `value`
        let mut i = 0;
        let start_ptr = self.buf.ptr.add::<T>(self.len);
        while i + self.len < new_len {
            start_ptr.add::<T>(i).write::<T>(value);
            i += 1;
        }

        self.len = new_len;
    }

    /// Returns the last element in the `Vec`.
    ///
    /// # Returns
    ///
    /// [Option<T>] - The last element in the `Vec` or `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let mut vec = Vec::new();
    ///     assert(vec.last() == None);
    ///     vec.push(1u64);
    ///     assert(vec.last() == Some(1u64));
    ///     vec.push(2u64);
    ///     assert(vec.last() == Some(2u64));
    /// }
    /// ```
    pub fn last(self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        Some(self.buf.ptr.add::<T>(self.len - 1).read::<T>())
    }
}

impl<T> AsRawSlice for Vec<T> {
    fn as_raw_slice(self) -> raw_slice {
        raw_slice::from_parts::<T>(self.buf.ptr, self.len)
    }
}

impl<T> From<raw_slice> for Vec<T> {
    fn from(slice: raw_slice) -> Self {
        Self {
            buf: RawVec::from(slice),
            len: slice.len::<T>(),
        }
    }
}

impl<T> From<Vec<T>> for raw_slice {
    fn from(vec: Vec<T>) -> Self {
        asm(ptr: (vec.buf.ptr, vec.len)) {
            ptr: raw_slice
        }
    }
}

impl<T> AbiEncode for Vec<T>
where
    T: AbiEncode,
{
    fn is_memcopy() -> bool {
        false
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let len = self.len;
        let mut buffer = len.abi_encode(buffer);

        let mut i = 0;
        while i < len {
            let item = self.get_unchecked(i);
            buffer = item.abi_encode(buffer);
            i += 1;
        }

        buffer
    }
}

impl<T> AbiDecode for Vec<T>
where
    T: AbiDecode,
{
    fn is_memcopy() -> bool {
        false
    }

    fn abi_decode(ref mut buffer: BufferReader) -> Vec<T> {
        let len = u64::abi_decode(buffer);

        let mut v = Vec::with_capacity(len);

        let mut i = 0;
        while i < len {
            let item = T::abi_decode(buffer);
            v.push(item);
            i += 1;
        }

        v
    }
}

pub struct VecIter<T> {
    values: Vec<T>,
    index: u64,
}

impl<T> Iterator for VecIter<T> {
    type Item = T;
    fn next(ref mut self) -> Option<Self::Item> {
        // BEWARE: `self.values` keeps **the copy** of the `Vec`
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

impl<T> Clone for Vec<T> {
    fn clone(self) -> Self {
        let len = self.len;
        let buf = RawVec::with_capacity(len);
        if len > 0 {
            self.buf.ptr.copy_to::<T>(buf.ptr, len);
        }
        Self { buf, len }
    }
}

impl<T> PartialEq for Vec<T>
where
    T: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        if self.len != other.len {
            return false;
        }
        let mut i = 0;
        while i < self.len {
            if self.get_unchecked(i) != other.get_unchecked(i) {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl<T> Debug for Vec<T>
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut l = f.debug_list();

        for elem in self.iter() {
            let _ = l.entry(elem);
        }

        l.finish();
    }
}
