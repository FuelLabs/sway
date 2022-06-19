library vec;

use ::alloc::{alloc, realloc};
use ::intrinsics::size_of;
use ::hash::sha256;
use ::mem::{read, write};
use ::option::Option;
use ::result::Result;
use ::storage::{store, get};

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

enum StorageVecError {
    IndexOutOfBounds: (),
}

/// A persistant vector struct
pub struct StorageVec<V> {}

impl<V> StorageVec<V> {
    /// Appends the value to the end of the vector
    #[storage(read, write)]
    fn push(self, value: V) {
        // The length of the vec is stored in the __get_storage_key() slot
        let len = get::<u64>(__get_storage_key());
        
        // Storing the value at the current length index (if this is the first item, starts off at 0)
        let key = sha256((len, __get_storage_key()));
        store::<V>(key, value);

        // Incrementing the length
        store(__get_storage_key(), len + 1);
    }

    /// Removes the last element of the vector
    #[storage(read, write)]
    fn pop(self) -> Option<V> {
        let len = get::<u64>(__get_storage_key());
        // if the length is 0, there is no item to pop from the vec
        if len == 0 {
            return Option::None;
        }
    
        // reduces len by 1, effectively removing the last item in the vec
        store(__get_storage_key(), len - 1);

        let key = sha256((len, __get_storage_key()));
        Option::Some(get::<V>(key))
    }

    /// Gets the value in the given index
    #[storage(read)]
    fn get(self, index: u64) -> Option<V> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to return
        if len <= index {
            return Option::None;
        }

        let key = sha256((index, __get_storage_key()));
        Option::Some(get::<V>(key))
    }

    /// Removes the value in the given index and moves all the values in the following indexes
    /// Down one index
    /// WARNING: Expensive for larger vecs
    #[storage(read, write)]
    fn remove_index(self, index: u64) -> Result<StorageVecError, V> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to remove
        if len <= index {
            return Result::Err(StorageVecError::IndexOutOfBounds);
        }

        // gets the element before removing it, so it can be returned
        let element_to_be_removed = get::<V>(sha256((index, __get_storage_key())));

        // for every element in the vec with an index greater than the input index,
        // shifts the index for that element down one
        let mut count = index + 1;
        while count < len {
            // gets the storage location for the previous index
            let key = sha256((count - 1, __get_storage_key()));
            // moves the element of the current index into the previous index
            store::<V>(key, get::<V>(sha256((count, __get_storage_key()))));
            
            count += 1;
        }

        // decrements len by 1
        store(__get_storage_key(), len - 1);

        // returns the removed element
        Result::Ok(element_to_be_removed)
    }

    /// Inserts the value at the given index, moving the current index's value aswell as the following's
    /// Up one index
    /// WARNING: Expensive for larger vecs
    #[storage(read, write)]
    fn insert(self, index: u64, value: V) -> Result<StorageVecError, ()> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no space to insert
        if index >= len {
            return Result::Err(StorageVecError::IndexOutOfBounds);
        }

        // for every element in the vec with an index larger than the input index,
        // move the element up one index.
        // performed in reverse to prevent data overwriting
        let mut count = len;
        while count > index {
            let key = sha256((count + 1, __get_storage_key()));
            // shifts all the values up one index
            store::<V>(key, get::<V>(sha256((count, __get_storage_key()))));

            count -= 1
        }

        // inserts the value into the now unused index
        let key = sha256((index, __get_storage_key()));
        store::<V>(key, value);

        // increments len by 1
        store(__get_storage_key(), len + 1);
        Result::Ok(())
    }
}
