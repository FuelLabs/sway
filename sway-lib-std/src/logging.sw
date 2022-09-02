//! Allows logging of arbitrary types, emitted as either `Log` or `Logd` receipts.
library logging;
use ::intrinsics::{is_reference_type, size_of};

/// Log any stack type.
/// If the type is a reference type, `log` is used.
/// Otherwise `logd` is used.'
pub fn log<T>(value: T) {
   __log::<T>(value); 
}
