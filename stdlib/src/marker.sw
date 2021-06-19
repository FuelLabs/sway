library marker;

//! The `marker` library represents traits that signify some fundamental safety intrinsic 
//! about some types, typically used for flagging types as safe for certain operations.

/// `Sized` denotes that a type has some known size at compile time, and is used to determine the
/// size of heap memory allocations.
pub trait Sized {
  /// Returns the size of this type in bytes.
  fn size_of() -> u64;
}

