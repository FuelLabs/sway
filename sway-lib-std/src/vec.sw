library vec;

use ::context::registers::heap_ptr;
use ::option::Option;

struct RawVec<T> {
    ptr: u64,
    cap: u64,
}

impl<T> RawVec<T> {
    /// Create a new `RawVec` with zero capacity.
    fn new() -> Self {
        RawVec {
            // Heap pointer points to _unallocated_ memory.
            ptr: heap_ptr() + 1,
            cap: 0,
        }
    }

    /// Creates a `RawVec` (on the heap) with exactly the capacity for a
    /// `[T; capacity]`. This is equivalent to calling `RawVec::new` when
    /// `capacity` is `0`.
    fn with_capacity(capacity: u64) -> Self {
        asm(size: capacity * __size_of::<T>()) {
            aloc size;
        };
        RawVec {
            // Heap pointer points to _unallocated_ memory.
            ptr: heap_ptr() + 1,
            cap: capacity,
        }
    }

    /// Gets the pointer of the allocation.
    fn ptr(self) -> u64 {
        self.ptr
    }

    /// Gets the capacity of the allocation.
    fn capacity(self) -> u64 {
        self.cap
    }

    fn grow(mut self) {
        let new_cap = if self.cap == 0 {
            1
        } else {
            2 * self.cap
        };

        // Allocate for `new_cap` elements.
        asm(size: new_cap * __size_of::<T>()) {
            aloc size;
        };
        let new_ptr = heap_ptr() + 1;

        // Copy old contents into newly-allocated memory.
        let copy_size = self.cap * __size_of::<T>();
        if copy_size > 0 {
            asm(new_ptr: new_ptr, old_ptr: self.ptr, size: copy_size) {
                mcp new_ptr old_ptr size;
            };
        }

        self.ptr = new_ptr;
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
    pub fn new() -> Self {
        Vec {
            buf: ~RawVec::new(),
            len: 0,
        }
    }

    /// Constructs a new, empty `Vec<T>` with the specified capacity.
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// *capacity* specified, the vector will have a zero *length*.
    pub fn with_capacity(capacity: u64) -> Self {
        Vec {
            buf: ~RawVec::with_capacity(capacity),
            len: 0,
        }
    }

    /// Appends an element to the back of a collection.
    pub fn push(mut self, value: T) {
        // If there is insufficient capacity, grow the buffer.
        if self.len == self.buf.capacity() {
            self.buf.grow();
        };
        // Get a pointer to the end of the buffer, where the new element will
        // be inserted.
        let end = self.buf.ptr() + self.len * __size_of::<T>();

        if !__is_reference_type::<T>() {
            // If `T` is not a reference type, then it is a one-word primitive.
            // Simply store it at `end` pointer.
            asm(end: end, value: value) {
                sw end value i0;
            };
        } else {
            // If `T` is a reference type, then it points to a memory range.
            // Copy the `value`'s memory range to the range pointed to by `end`.
            let size = __size_of::<T>();
            asm(end: end, ptr: value, size: size) {
                mcp end ptr size;
            };
        };

        // Increment length.
        self.len = self.len + 1;
    }

    /// Gets the capacity of the allocation.
    pub fn capacity(self) -> u64 {
        self.buf.cap
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the vector.
    pub fn clear(mut self) {
        self.len = 0;
    }

    /// Returns a copy or reference to an element.
    ///
    /// - If `T` is a copy type then returns a copy of the element, or `None` if
    ///   out of bounds.
    /// - If `T` is a reference type then returns a reference to the element at
    ///   that position or `None` if out of bounds.
    ///
    /// Note that since a reference is returned, mutating the returned value
    /// without affecting the underlying vector requires a copy.
    pub fn get(self, index: u64) -> Option<T> {
        // First check that index is within bounds.
        if index >= self.len {
            return Option::None::<T>();
        };

        let ptr = self.buf.ptr() + index * __size_of::<T>();

        if !__is_reference_type::<T>() {
            // If `T` is not a reference type, then it is a one-word primitive.
            // Simply store it at `end` pointer.
            let res = asm(res, ptr: ptr) {
                lw res ptr i0;
                res: T
            };
            Option::Some(res)
        } else {
            // If `T` is a reference type, then it points to a memory range.
            // Copy the `value`'s memory range to the range pointed to by `end`.
            let res = asm(ptr: ptr) {
                ptr: T
            };
            Option::Some(res)
        }
    }

    /// Returns the number of elements in the vector, also referred to
    /// as its 'length'.
    pub fn len(self) -> u64 {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(self) -> bool {
        self.len == 0
    }
}
