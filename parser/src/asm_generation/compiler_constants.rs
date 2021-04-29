//! This module contains constants that are used in the compiler but are not necessarily inherent
//! to the VM.
//!
// Rustfmt is set to skip this file so we can see the bytes all lined up.

#![allow(dead_code)]
#[rustfmt::skip]
pub(crate) mod registers {
    use crate::asm_lang::RegisterId; 
    pub(crate) const FALSE: RegisterId = RegisterId::Constant(0x00);
    pub(crate) const ZERO:  RegisterId = RegisterId::Constant(0x00);
    pub(crate) const TRUE:  RegisterId = RegisterId::Constant(0x01);
}
