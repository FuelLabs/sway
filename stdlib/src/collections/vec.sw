library vec;
use ::ops::*;
use ::marker::Sized;

/// If `RawVec` is exceeded during a push, the size of the vector is doubled 
pub struct Vec<T> {
  buf: RawVec,
  len: u64
}

/// Contains the `ptr` to the start of the vector and the current length of the vector
/// in bytes
/// Basically a wrapper over the return value of `aloc`
struct RawVec {
  ptr: u64,
  size: u64,
}

impl<T> Vec<T> where T: Sized {
  /// Creates a new empty vector with enough size for 1 element of type `T`.
  /// The size of the vector's underlying memory buffer doubles each time its 
  /// capacity is exceeded.
  /// If you know how big the buffer should be, use `Vec::with_capacity` instead.
  fn new() -> Self {
    let item_size = ~T::heap_size_of();
    Vec { buf: ~RawVec::new(item_size), len: 0 }
  }

  /// Initializes a new vector with a pre-determined capacity allocated.
  /// If you know the size the vector will end up being, you should initialize
  /// your vector with this method instead, to save on future `aloc` calls.
  fn with_capacity(capacity: u64) -> Self {
    let capacity_bytes = capacity.multiply(~T::heap_size_of());
    Vec { buf: ~RawVec::new(capacity_bytes), len: 0 }
  }

  /// Push an item on to the end of the vector.
  fn push(self, item: T) {
    let size_of_item = ~T::heap_size_of();
    // If this item would exceed the boundaries of the underlying buffer, we
    // need allocate a new, bigger buffer. 
    // TODO nested struct field
    if (self.len.multiply(size_of_item)) + size_of_item > self.buf.size {
          // allocate a new buffer, copy the old contents over, set self.buf = new buf
    } else {
          // write T to self.len * size_of_item
          // increase self.len by one
    }
  }
}


impl RawVec {
  fn new(init_size_bytes: u64) -> Self {
    let ptr = asm(r1: init_size_bytes) {
      aloc r1;
      hp: u64
    };

    RawVec {
      ptr: ptr,
      size: init_size_bytes,
    }
  }
}

