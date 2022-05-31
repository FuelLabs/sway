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

/// Returns the size of a value in bytes.
pub fn size_of_val<T>(val: T) -> u64 {
    __size_of_val::<T>(val)
}

/// Returns the address of the given value.
pub fn addr_of<T>(val: T) -> u64 {
    // TODO: Replace with intrinsic: https://github.com/FuelLabs/sway/issues/855
    if !__is_reference_type::<T>() {
        // std::revert not available here
        asm() {
            rvrt zero;
        }
    }
    asm(ptr: val) {
        ptr: u64
    }
}

/// Copies data from source to destination.
pub fn copy(dst: u64, src: u64, size: u64) {
    // TODO: Replace with intrinsic: https://github.com/FuelLabs/sway/issues/855
    asm(dst: dst, src: src, size: size) {
        mcp dst src size;
    };
}

/// Compares data at two points of memory.
pub fn raw_eq(first: u64, second: u64, len: u64) -> bool {
    // TODO: Replace with intrinsic: https://github.com/FuelLabs/sway/issues/855
    asm(first: first, second: second, len: len, result) {
        meq result first second len;
        result: bool
    }
}
