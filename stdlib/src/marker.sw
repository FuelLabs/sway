library marker;

//! The `marker` library represents traits that signify some fundamental safety intrinsic 
//! about some types, typically used for flagging types as safe for certain operations.

/// `Sized` denotes that a type has some known size at runtime, and is used to determine the
/// size of heap memory allocations.
pub trait Sized {
  /// Returns the size of this type on the heap in bytes.
  fn heap_size_of() -> u64;
}

