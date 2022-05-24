//! Allows logging of arbitrary types, emitted as either `Log` or `Logd` receipts.
library logging;
use ::intrinsics::{is_reference_type, size_of};

/// Log any stack type.
/// If the type is a reference type, `log` is used.
/// Otherwise `logd` is used.'
pub fn log<T>(value: T) {
    if !is_reference_type::<T>() {
        asm(r1: value) {
            log r1 zero zero zero;
        }
    } else {
        let size = size_of::<T>();
        asm(r1: value, r2: size) {
            logd zero zero r1 r2;
        };
    }
}
