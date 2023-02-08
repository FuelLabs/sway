library vec;

use core::ops::{Eq, Ord};
use ::alloc::{alloc, realloc};
use ::assert::assert;
use ::option::Option;
use ::convert::From;

struct RawVec<T> {
    ptr: raw_ptr,
    cap: u64,
}

impl<T> RawVec<T> {
    /// Create a new `RawVec` with zero capacity.
    pub fn new() -> Self {
        Self {
            ptr: alloc::<T>(0),
            cap: 0,
        }
    }

    /// Creates a `RawVec` (on the heap) with exactly the capacity for a
    /// `[T; capacity]`. This is equivalent to calling `RawVec::new` when
    /// `capacity` is zero.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            ptr: alloc::<T>(capacity),
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

    /// Grow the capacity of the vector by doubling its current capacity. The
    /// `realloc` function allocates memory on the heap and copies the data
    /// from the old allocation to the new allocation.
    pub fn grow(ref mut self) {
        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };

        self.ptr = realloc::<T>(self.ptr, self.cap, new_cap);
        self.cap = new_cap;
    }
}

/// A contiguous growable array type, written as `Vec<T>`, short for 'vector'.
pub struct Vec<T> {
    buf: RawVec<T>,
    len: u64,
}

impl<T> Vec<T> {
    /// Constructs a new, empty `Vec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// // allocates when an element is pushed
    /// vec.push(5);
    /// ```
    pub fn new() -> Self {
        Self {
            buf: RawVec::new(),
            len: 0,
        }
    }

    /// Constructs a new, empty `Vec<T>` with the specified capacity.
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is zero, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// *capacity* specified, the vector will have a zero *length*.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::with_capacity(2);
    /// // does not allocate
    /// vec.push(5);
    /// // does not re-allocate
    /// vec.push(10);
    /// ```
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            buf: RawVec::with_capacity(capacity),
            len: 0,
        }
    }

    /// Appends an element to the back of a collection.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// let last_element = vec.pop().unwrap();
    /// assert(last_element == 5);
    /// ```
    pub fn push(ref mut self, value: T) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.capacity() {
            self.buf.grow();
        };

        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        let end = self.buf.ptr().add::<T>(self.len);

        // Write `value` at pointer `end`
        end.write::<T>(value);

        // Increment length.
        self.len += 1;
    }

    /// Gets the capacity of the allocation.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::with_capacity(5);
    /// let cap = vec.capacity();
    /// assert(cap == 5);
    /// ```
    pub fn capacity(self) -> u64 {
        self.buf.cap
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the vector.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// vec.clear()
    /// assert(vec.is_empty());
    /// ```
    pub fn clear(ref mut self) {
        self.len = 0;
    }

    /// Returns a vector element at `index`, or `None` if `index` is out of
    /// bounds.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// vec.push(10);
    /// vec.push(15);
    /// let item = vec.get(1).unwrap();
    /// assert(item == 10);
    /// let res = vec.get(10);
    /// assert(res.is_none()); // index out of bounds
    /// ```
    pub fn get(self, index: u64) -> Option<T> {
        // First check that index is within bounds.
        if self.len <= index {
            return Option::None;
        };

        // Get a pointer to the desired element using `index`
        let ptr = self.buf.ptr().add::<T>(index);

        // Read from `ptr`
        Option::Some(ptr.read::<T>())
    }

    /// Returns the number of elements in the vector, also referred to
    /// as its `length`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// assert(vec.len() == 1);
    /// vec.push(10);
    /// assert(vec.len() == 2);
    /// ```
    pub fn len(self) -> u64 {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// assert(vec.is_empty());
    /// vec.push(5);
    /// assert(!vec.is_empty());
    /// vec.clear()
    /// assert(vec.is_empty());
    /// ```
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// ### Reverts
    ///
    /// * If `index >= self.len`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// vec.push(10);
    /// vec.push(15);
    /// let item = vec.remove(1);
    /// assert(item == 10);
    /// assert(vec.get(0).unwrap() == 5);
    /// assert(vec.get(1).unwrap() == 15);
    /// assert(vec.get(2).is_none());
    /// ```
    pub fn remove(ref mut self, index: u64) -> T {
        assert(index < self.len);

        let buf_start = self.buf.ptr();

        // Read the value at `index`
        let ptr = buf_start.add::<T>(index);
        let ret = ptr.read::<T>();

        // Shift everything down to fill in that spot.
        let mut i = index;
        while i < self.len {
            let ptr = buf_start.add::<T>(i);
            ptr.add::<T>(1).copy_to::<T>(ptr, 1);
            i += 1;
        }

        // Decrease length.
        self.len -= 1;
        ret
    }

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right.
    /// Panics if `index > len`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// vec.push(10);
    ///
    /// vec.insert(1, 15);
    ///
    /// assert(vec.get(0).unwrap() == 5);
    /// assert(vec.get(0).unwrap() == 15);
    /// assert(vec.get(0).unwrap() == 10);
    /// ```
    pub fn insert(ref mut self, index: u64, element: T) {
        assert(index <= self.len);

        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.cap {
            self.buf.grow();
        }

        let buf_start = self.buf.ptr();

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

    /// Removes the last element from a vector and returns it, or `None` if it
    /// is empty.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    ///
    /// let res = vec.pop();
    /// assert(res.is_none());
    ///
    /// vec.push(5);
    /// let res = vec.pop();
    /// assert(res.unwrap() == 5);
    /// assert(vec.is_empty());
    /// ```
    pub fn pop(ref mut self) -> Option<T> {
        if self.len == 0 {
            return Option::None;
        }
        self.len -= 1;
        Option::Some(self.buf.ptr().add::<T>(self.len).read::<T>())
    }

    /// Swaps two elements.
    ///
    /// ### Arguments
    ///
    /// * element1_index - The index of the first element.
    /// * element2_index - The index of the second element.
    ///
    /// ### Reverts
    ///
    /// * If `element1_index` or `element2_index` is greater than or equal to the length of vector.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// vec.push(10);
    ///
    /// vec.swap(0, 1);
    ///
    /// assert(vec.get(0).unwrap() == 10);
    /// assert(vec.get(1).unwrap() == 5);
    /// ```
    pub fn swap(ref mut self, element1_index: u64, element2_index: u64) {
        assert(element1_index < self.len);
        assert(element2_index < self.len);

        if element1_index == element2_index {
            return;
        }

        let element1_ptr = self.buf.ptr().add::<T>(element1_index);
        let element2_ptr = self.buf.ptr().add::<T>(element2_index);

        let element1_val: T = element1_ptr.read::<T>();
        element2_ptr.copy_to::<T>(element1_ptr, 1);
        element2_ptr.write::<T>(element1_val);
    }

    /// Updates an element at position `index` with a new element `value`.
    ///
    /// ### Arguments
    ///
    /// * index - The index of the element to be set.
    /// * value - The value of the element to be set.
    ///
    /// ### Reverts
    ///
    /// * If `index` is greater than or equal to the length of vector.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// vec.push(10);
    ///
    /// vec.set(0, 15);
    ///
    /// assert(vec.get(0).unwrap() == 15);
    /// assert(vec.get(1).unwrap() == 10);
    /// ```
    pub fn set(ref mut self, index: u64, value: T) {
        assert(index < self.len);

        let index_ptr = self.buf.ptr().add::<T>(index);

        index_ptr.write::<T>(value);
    }


    /// Moves all elements of `other` into `self`, leaving `other` empty.
    ///
    /// ### Arguments
    ///
    /// * other - The vector to append to self
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// let mut vec2 = Vec::new();
    /// vec2.push(3);
    /// vec2.push(4);
    ///
    /// vec.append(vec2);
    ///
    /// assert(vec.get(0).unwrap() == 1);
    /// assert(vec.get(1).unwrap() == 2);
    /// assert(vec.get(2).unwrap() == 3);
    /// assert(vec.get(3).unwrap() == 4);
    /// ```
    pub fn append(ref mut self, ref other: Vec<T>) {
        let combined_len = self.len + other.len;

        // if combined length exceeds capacity, reallocate with capacity of combined length
        if self.buf.cap < combined_len {
            self.buf.ptr = realloc::<T>(self.buf.ptr, self.buf.cap, combined_len);
        }

        // write to buffer
        other.buf.ptr.copy_to::<T>(self.buf.ptr.add::<T>(self.len), other.len);

        // set lengths and capacity
        self.len = combined_len;
        self.buf.cap = combined_len;
        other.len = 0;
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated vector containing the elements at the range
    /// `[at, len)`. After the call, the original vector will be left containing
    /// the elements `[0, at)` with its previous capacity unchanged.
    ///
    /// ### Arguments
    ///
    /// * at - Index at which the vector is to be split
    ///
    /// ### Reverts
    ///
    /// * if `at > self.len`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// let mut vec2 = vec.split_off(1);
    ///
    /// assert(vec.get(0).unwrap() == 1);
    /// assert(vec2.get(0).unwrap() == 2);
    /// assert(vec2.get(1).unwrap() == 3);
    /// ```
    pub fn split_off(ref mut self, at: u64) -> Vec<T> {
        assert(self.len >= at);

        let split_len = self.len - at;
        let mut split_vec = Self { buf: RawVec::with_capacity(split_len), len: split_len };

        self.buf.ptr().add::<T>(at).copy_to::<T>(split_vec.buf.ptr(), split_len);

        self.len = at;

        split_vec
    }

    /// Divides one vector into two at an index.
    ///
    /// The first will contain all indices from `[0, mid)` (excluding the index
    /// `mid` itself) and the second will contain all indices from `[mid, len)`
    /// (excluding the index `len` itself).
    ///
    /// ### Arguments
    ///
    /// * mid - Index at which the vector is to be split
    ///
    /// ### Reverts
    ///
    /// * if `mid > self.len`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// let (left, right) = vec.split_at(2);
    ///
    /// assert(left.get(0).unwrap() == 1);
    /// assert(left.get(1).unwrap() == 2);
    ///
    /// assert(right.get(0).unwrap() == 3);
    /// assert(right.get(1).unwrap() == 4);
    /// ```
    pub fn split_at(self, mid: u64) -> (Vec<T>, Vec<T>) {
        assert(self.len >= mid);

        let left_len = mid;
        let right_len = self.len - mid;

        let mut left_vec = Self { buf: RawVec::with_capacity(left_len), len: left_len };
        let mut right_vec = Self { buf: RawVec::with_capacity(right_len), len: right_len };

        self.buf.ptr.copy_to::<T>(left_vec.buf.ptr, left_len);
        self.buf.ptr.add::<T>(mid).copy_to::<T>(right_vec.buf.ptr, right_len);

        left_vec.len = left_len;
        right_vec.len = right_len;

        (left_vec, right_vec)
    }

    /// Returns the first element of the vector, or `None` if it is empty.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// let mut vec2 = Vec::new();
    ///
    /// assert(vec.first().unwrap() == 1);
    /// assert(vec2.first().is_none());
    /// ```
    pub fn first(self) -> Option<T> {
        match self.len {
            0 => Option::None,
            _ => Option::Some(self.buf.ptr.read::<T>()),
        }
    }

    /// Returns the last element of the vector, or `None` if it is empty.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(1); 
    /// vec.push(2);
    ///
    /// let mut vec2 = Vec::new();
    ///
    /// assert(vec.last().unwrap() == 2);
    /// assert(vec2.last().is_none());
    /// ```
    pub fn last(self) -> Option<T> {
        match self.len {
            0 => Option::None,
            n => Option::Some(self.buf.ptr().add::<T>(n - 1).read::<T>()),
        }
    }

    /// Reverses the order of elements in the vector, in place.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut v = Vec::new();
    /// v.push(1);
    /// v.push(2);
    /// v.push(3);
    /// v.reverse();
    ///
    /// assert(v.get(0.unwrap() == 3);
    /// assert(v.get(1).unwrap() == 2);
    /// assert(v.get(2).unwrap() == 1);
    /// ```
    pub fn reverse(ref mut self) {
        let len = self.len;

        if len >= 2 {
            let mut i = 0;
            while i < len / 2 {
                let element1_ptr = self.buf.ptr.add::<T>(i);
                let element2_ptr = self.buf.ptr.add::<T>(len - i - 1);

                let element1_value: T = element1_ptr.read::<T>();
                element2_ptr.copy_to::<T>(element1_ptr, 1);
                element2_ptr.write::<T>(element1_value);

                i += 1;
            }
        }
    }

    /// Fills `self` by with elements by cloning `value`.
    ///
    /// ### Arguments
    ///
    /// * value - Value to copy to each element of the vector
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(0);
    /// vec.push(0);
    /// vec.push(0);
    ///
    /// vec.fill(1);
    ///
    /// assert(vec.get(0).unwrap() == 1);
    /// assert(vec.get(1).unwrap() == 1);
    /// assert(vec.get(2).unwrap() == 1);
    /// ```
    pub fn fill(ref mut self, value: T) {
        let mut i = 0;
        while i < self.len {
            self.buf.ptr.add::<T>(i).write::<T>(value);
            i += 1;
        }
    }

    /// Returns `true` if the vector contains an element with the given `value`.
    ///
    /// This operation is O(n).
    ///
    /// ### Arguments
    ///
    /// * value - The value to check
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::vec::Vec;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// assert(vec.contains(2));
    /// assert(!vec.contains(3));
    /// ```
    // TODO: replace `v` with `self` when trait constraints are merged
    pub fn contains<O>(ref mut v: Vec<O>, value: O) -> bool
    where
        O: Eq
    {
        let mut i = 0;
        while i < v.len {
            let index_ptr = v.buf.ptr.add::<O>(i);
            if value == index_ptr.read::<O>() {
                return true;
            }
            i += 1;
        }
        return false;
    }

    /// Resizes the `Vec` in place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the difference, with each
    /// additional slot filled with `value`. If the `new_len` is greater than the capacity, it is
    /// reallocated on the heap. If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// ### Arguments
    ///
    /// new_len - The new length to expand or truncate to
    /// value - The value to fill into new slots if the `new_len` is greater than the current length
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(0);
    /// vec.fill(3, 1);
    ///
    /// let mut vec2 = Vec::new();
    /// vec2.push(1);
    /// vec2.push(2);
    /// vec.fill(1, 3);
    ///
    /// assert(vec.len() == 3);
    /// assert(vec.get(0).unwrap() == 0);
    /// assert(vec.get(1).unwrap() == 1);
    /// assert(vec.get(2).unwrap() == 1);
    ///
    /// assert(vec2.len() == 1);
    /// assert(vec2.get(0).unwrwap() == 1);
    /// ```
    pub fn resize(ref mut self, new_len: u64, value: T) {
        let len = self.len;

        if new_len <= len {
            self.len = new_len;
            return;
        }

        if new_len > self.buf.cap {
            // need to reallocate
            self.buf.ptr = realloc::<T>(self.buf.ptr, self.buf.cap, new_len);
            self.buf.cap = new_len;
        }

        let mut i = len;
        while i < new_len {
            self.buf.ptr.add::<T>(i).write::<T>(value);
            i += 1;
        }

        self.len = new_len;
    }

    /// Checks if the elements of the vector are sorted.
    ///
    /// That is, for every element `a` and its following element `b`, `a <= b` must hold. If the
    /// vector holds zero or one element, `true` is returned.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// assert(vec.is_sorted());
    ///
    /// vec.swap(0, 1);
    ///
    /// assert(!vec.is_sorted());
    /// ```
    pub fn is_sorted<O>(ref mut v: Vec<O>) -> bool where O: Ord {
        let len = v.len;

        if len < 2 {
            return true;
        }

        let mut i = 0;
        while i < len - 1 {
            let current = v.buf.ptr.add::<O>(i).read::<O>();
            let next = v.buf.ptr.add::<O>(i + 1).read::<O>();
            if current > next {
                return false;
            }
            i += 1;
        }

        true
    }
}

impl<T> Vec<T> {
    /// Iterative Insertion Sort
    ///
    /// - Average case time complexity O(n ^ 2)
    /// - Space complexity O(1) auxilary
    // TODO: replace `v` with `self` when trait constraints are merged
    fn insertion_sort<O>(ref mut v: Vec<O>) where O: Ord + Eq {
        let len = v.len;
        if 2 > len {
            return;
        }
        let mut i = 0;
        while i < len {
            let mut j = i;
            while j > 0 && v.get(j).unwrap() < v.get(j - 1).unwrap() {
                v.swap(j, j - 1);
                j -= 1;
            }
            i += 1;
        }
    }

    /// Iterative Quick Sort
    ///
    /// Average case time complexity TODO
    /// Space complexity O(n) auxillary
    // TODO: replace `v` with `self` when trait constraints are merged
    fn quick_sort<O>(ref mut v: Vec<O>) where O: Ord + Eq {
        let len = v.len;

        if 2 > len {
            return;
        }

        let mut low = 0;
        let mut high = len - 1;

        // Allocate auxillary stack.
        let mut stack = Vec::with_capacity(len);

        // Push low and high to stack.
        stack.push(low);
        stack.push(high);

        // Loop while stack not empty.
        while stack.len() > 0 {
            // Pop high and low.
            high = stack.pop().unwrap();
            low = stack.pop().unwrap();

            // Partition step.
            let mut pivot = v.get(high).unwrap();
            let mut partition_index = low;
            let mut i = low;
            while i < high {
                let i_element = v.get(i).unwrap();
                // TODO: refactor when OrdEq is exposed.
                if i_element < pivot || i_element == pivot {
                    v.buf.ptr().add::<O>(i).write::<O>(v.get(partition_index).unwrap());
                    v.buf.ptr().add::<O>(partition_index).write::<O>(i_element);
                    partition_index += 1;
                }
                i += 1;
            }
            let temp = v.get(partition_index).unwrap();
            v.buf.ptr().add::<O>(partition_index).write::<O>(pivot);
            v.buf.ptr().add::<O>(high).write::<O>(temp);

            // If elements to the left of the pivot, push the left side.
            if partition_index > low + 1 {
                stack.push(low);
                stack.push(partition_index - 1);
            }

            // If elements to the right of the pivot, push the right side.
            if partition_index + 1 < high {
                stack.push(partition_index + 1);
                stack.push(high);
            }
        }
    }
}

impl<T> Vec<T> {
    /// Iterative Merge Sort
    ///
    /// - Average case time complexity O(n log n) 
    /// - Space complexity O(2 n) auxilary
    // TODO: replace `v` with `self` when trait constraints are merged
    fn merge_sort<O>(ref mut v: Vec<O>) where O: Ord + Eq {
        // The Rust implementation is insertionn sort if len <= 20, else modified timsort. However,
        // the Rust implementation is based on their own benchmarks. We will need to run benchmarks
        // with Fuel VM gas and optimize as needed.
        const MAX_INSERTION: u64 = 21;

        let len = v.len;

        if len < 2 {
            return;
        }

        // If length is 20 or less, use insertion sort instead.
        if len < MAX_INSERTION {
            Vec::insertion_sort(v);
            return;
        }

        // Buffer for merging.
        let mut buffer = alloc::<O>(len);

        // Temporary memory for swapping `v` and `buffer` in each chunk iteration.
        let mut swap = alloc::<O>(len);

        // `chunk_start` starts at 1 and doubles in size each iteration.
        let mut chunk_size = 1;
        while chunk_size < len {
            // `left_start` begins at the first chunk and iterates each chunk.
            let mut left_start = 0;
            while left_start < len {
                // Merge logic.
                let mut left = left_start;
                let mut right = if left + chunk_size > len { len } else { left + chunk_size };
                let left_limit = right;
                let right_limit = if right + chunk_size > len { len } else { right + chunk_size };

                // Left subvector = vec[left_start..left_limit]
                // Right subvector = vec[right_start..right_limit]

                // Loop over the left and right subvectors until the shorter is "consumed", sorting
                // in the process
                let mut i = left;
                while left < left_limit && right < right_limit {
                    let left_element = v.get(left).unwrap();
                    let right_element = v.get(right).unwrap();

                    // TODO: refactor when OrdEq is exposed
                    if left_element < right_element || left_element == right_element {
                        buffer.add::<O>(i).write::<O>(left_element);
                        left += 1;
                    } else {
                        buffer.add::<O>(i).write::<O>(right_element);
                        right += 1;
                    }

                    i += 1;
                }

                // If the left subvector was longer, consume the remaining elements
                while left < left_limit {
                    buffer.add::<O>(i).write::<O>(v.get(left).unwrap());
                    left += 1;
                    i += 1;
                }

                // If the right subvector was longer, consume the remaining elements
                while right < right_limit {
                    buffer.add::<O>(i).write::<O>(v.get(right).unwrap());
                    right += 1;
                    i += 1;
                }

                // Increment the `left_start` by the chunk size times two
                left_start += (2 * chunk_size);
            }

            // `swap = buffer`
            // `buffer = vec`
            // `vec = swap`
            buffer.copy_to::<O>(swap, len);
            v.buf.ptr().copy_to::<O>(buffer, len);
            swap.copy_to::<O>(v.buf.ptr(), len);

            // Double the chunk size
            chunk_size *= 2;
        }
    }
}

impl<T> Vec<T> {
    /// Sorts the Vector.
    ///
    /// This sort is stable (i.e. does not reorder equal elements).
    ///
    /// The current algorithm conditionally sorts via insertion sort if the length is 20 or less,
    /// otherwise it uses bottom-up merge sort. Insertion sort has an average time complexity of
    /// O(n^2) with no auxillary memory and merge sort has an average time complexity of
    /// O(n log n) with O(2n) memory.
    ///
    /// > Notice: This implementation may change in the future.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(3);
    /// vec.push(2);
    /// vec.push(4);
    ///
    /// vec.sort();
    ///
    /// assert(vec.get(0).unwrap() == 2);
    /// assert(vec.get(1).unwrap() == 3);
    /// assert(vec.get(2).unwrap() == 4);
    /// ```
    pub fn sort<O>(ref mut v: Vec<O>) where O: Ord + Eq {
        Vec::merge_sort(v);
    }

    /// Sorts the vector, but might not preserve the order of equal elements.
    ///
    /// This sort is unstable (i.e. may reorder equal elements).
    ///
    /// The current algorithm is an iterative quicksort with an average time complexity of
    /// O(n log n). Note that, since the sorting implementation is iterative, it requires O(n)
    /// auxillary memory for the stack.
    ///
    /// > Notice: This implementation may change in the future.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    /// vec.push(3);
    /// vec.push(2);
    /// vec.push(4);
    ///
    /// vec.sort_unstable();
    ///
    /// assert(vec.get(0).unwrap() == 2);
    /// assert(vec.get(1).unwrap() == 3);
    /// assert(vec.get(2).unwrap() == 4);
    /// ```
    pub fn sort_unstable<O>(ref mut v: Vec<O>) where O: Ord + Eq {
        Vec::quick_sort(v);
    }
}

impl<T> AsRawSlice for Vec<T> {
    /// Returns a raw slice to all of the elements in the vector.
    fn as_raw_slice(self) -> raw_slice {
        raw_slice::from_parts::<T>(self.buf.ptr(), self.len)
    }
}

impl<T> From<raw_slice> for Vec<T> {
    fn from(slice: raw_slice) -> Vec<T> {
        let buf = RawVec {
            ptr: slice.ptr(),
            cap: slice.len::<T>(),
        };
        Self {
            buf,
            len: buf.cap,
        }
    }

    fn into(self) -> raw_slice {
        asm(ptr: (self.buf.ptr(), self.len)) { ptr: raw_slice }
    }
}
