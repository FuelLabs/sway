library vec;

use ::alloc::{alloc, realloc};
use ::intrinsics::size_of;
use ::mem::{read, write};
use ::option::Option;

struct RawVec<T> {
    ptr: u64,
    cap: u64,
}

impl<T> RawVec<T> {
    /// Create a new `RawVec` with zero capacity.
    fn new() -> Self {
        RawVec {
            ptr: alloc(0),
            cap: 0,
        }
    }

    /// Creates a `RawVec` (on the heap) with exactly the capacity for a
    /// `[T; capacity]`. This is equivalent to calling `RawVec::new` when
    /// `capacity` is `0`.
    fn with_capacity(capacity: u64) -> Self {
        RawVec {
            ptr: alloc(capacity * size_of::<T>()),
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

    /// Grow the capacity of the vector by doubling its current capacity. The
    /// `realloc` function / allocates memory on the heap and copies the data
    /// from the old allocation to the new allocation
    fn grow(mut self) {
        let new_cap = if self.cap == 0 {
            1
        } else {
            2 * self.cap
        };

        self.ptr = realloc(self.ptr, self.cap * size_of::<T>(), new_cap * size_of::<T>());
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
        let end = self.buf.ptr() + self.len * size_of::<T>();

        // Write `value` at pointer `end`
        write(end, value);

        // Increment length.
        self.len += 1;
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

    /// Returns a vector element at `index`, or None if `index` is out of
    /// bounds.
    pub fn get(self, index: u64) -> Option<T> {
        // First check that index is within bounds.
        if self.len <= index {
            return Option::None::<T>();
        };

        // Get a pointer to the desired element using `index`
        let ptr = self.buf.ptr() + index * size_of::<T>();

        // Read from `ptr`
        Option::Some(read(ptr))
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
