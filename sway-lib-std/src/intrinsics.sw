//! Exposes compiler intrinsics as stdlib wrapper functions.
library intrinsics;

/// Returns whether a generic type `T` is a reference type or not.
pub fn is_reference_type<T>() -> bool {
    __is_reference_type::<T>()
}

/// Returns the size of a generic type `T` in bytes.
pub fn size_of<T>() -> u64 {
    __size_of::<T>()
}

///
pub fn get_storage_key() -> b256 {
    __get_storage_key()
}
