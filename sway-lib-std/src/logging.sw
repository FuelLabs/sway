//! Allows logging of arbitrary stack types, emitted as either `Log` or `Logd` receipts.
library;

use ::codec::*;
use ::debug::*;

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
#[cfg(experimental_new_encoding = false)]
pub fn log<T>(value: T) {
    __log::<T>(value);
}

#[cfg(experimental_new_encoding = true)]
pub fn log<T>(value: T)
where
    T: AbiEncode,
{
    __log::<T>(value);
}
