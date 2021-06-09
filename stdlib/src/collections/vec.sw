library vec;

/// If `RawVec` is exceeded during a push, the size of the vector is doubled 
pub struct Vec<T> {
  buf: RawVec,
  len: u32
}

/// Contains the `ptr` to the start of the vector and the current length of the vector
/// in terms of `T`. 
struct RawVec {
  ptr: u64,
  size: u64,
}

impl<T> Vec<T> where T: Sized {
  fn new() -> Self {
    let item_size = ~T::heap_size_of();
    Vec { buf: RawVec::new(item_size), len: 1 }
  }
}


impl RawVec {
  fn new(init_size_bytes: u64) -> Self {
    let ptr = asm(r1: init_size_bytes) {
      aloc r1;
      hp: u64
    };

  }
}

trait Sized {
  /// Returns the size of this type on the heap in bytes.
  fn heap_size_of() -> u64;
}
