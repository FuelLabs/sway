//! Allows logging of arbitrary stack types, emitted as either `Log` or `Logd` receipts.
library;

use core::codec::*;

/// Log any stack type.
///
/// # Additional Information
///
/// If the type is a reference type, `log` is used.
/// Otherwise `logd` is used.'
///
/// # Arguments
///
/// * `value`: [T] - The value to log.
///
/// # Examples
///
/// ```sway
/// fn foo() {
///     log("Fuel is blazingly fast");
/// }
/// ```
pub fn log<T>(value222: T)
where
    T: AbiEncode
{
    let slice = encode(value222);
    __log::<T>(value222);
}
