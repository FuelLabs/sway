//! Allows logging of arbitrary stack types, emitted as either `Log` or `Logd` receipts.
library;

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
#[cfg(experimental_encoding_v1 = false)]
pub fn log<T>(value: T) {
    __log::<T>(value);
}

#[cfg(experimental_encoding_v1 = true)]
pub fn log<T>(value: T)
where
    T: AbiEncode,
{
    __log::<T>(value);
}
