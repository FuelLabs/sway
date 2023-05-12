//! Allows logging of arbitrary stack types, emitted as either `Log` or `Logd` receipts.
library;

/// Log any stack type.
/// If the type is a reference type, `log` is used.
/// Otherwise `logd` is used.'
pub fn log<T>(value: T) {
    __log::<T>(value);
}
