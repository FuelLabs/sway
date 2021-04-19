//! This module contains constants that are used in the compiler but are not necessarily inherent
//! to the VM.
//!
// Rustfmt is set to skip this file so we can see the bytes all lined up.

#[rustfmt::skip]
pub(crate) mod registers {
    const FALSE: u64 = 0x00;
    const TRUE:  u64 = 0x01;
}
