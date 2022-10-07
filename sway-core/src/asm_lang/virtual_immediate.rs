use crate::error::*;
use sway_types::span::Span;

use std::convert::TryInto;
use std::fmt;

/// 6-bit immediate value type
#[derive(Clone, Debug)]
pub struct VirtualImmediate06 {
    pub(crate) value: u8,
}

impl VirtualImmediate06 {
    pub(crate) fn new(raw: u64, err_msg_span: Span) -> Result<Self, CompileError> {
        if raw > crate::asm_generation::compiler_constants::SIX_BITS {
            Err(CompileError::Immediate06TooLarge {
                val: raw,
                span: err_msg_span,
            })
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
}
impl fmt::Display for VirtualImmediate06 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}

/// 12-bits immediate value type
#[derive(Clone, Debug)]
pub struct VirtualImmediate12 {
    pub(crate) value: u16,
}

impl VirtualImmediate12 {
    pub(crate) fn new(raw: u64, err_msg_span: Span) -> Result<Self, CompileError> {
        if raw > crate::asm_generation::compiler_constants::TWELVE_BITS {
            Err(CompileError::Immediate12TooLarge {
                val: raw,
                span: err_msg_span,
            })
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().unwrap_or_else(|_| panic!("{}", msg.into())),
        }
    }
}
impl fmt::Display for VirtualImmediate12 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}

/// 18-bits immediate value type
#[derive(Clone, Debug)]
pub struct VirtualImmediate18 {
    pub(crate) value: u32,
}
impl VirtualImmediate18 {
    pub(crate) fn new(raw: u64, err_msg_span: Span) -> Result<Self, CompileError> {
        if raw > crate::asm_generation::compiler_constants::EIGHTEEN_BITS {
            Err(CompileError::Immediate18TooLarge {
                val: raw,
                span: err_msg_span,
            })
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().unwrap_or_else(|_| panic!("{}", msg.into())),
        }
    }
}
impl fmt::Display for VirtualImmediate18 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}

/// 24-bits immediate value type
#[derive(Clone, Debug)]
pub struct VirtualImmediate24 {
    pub(crate) value: u32,
}
impl VirtualImmediate24 {
    pub(crate) fn new(raw: u64, err_msg_span: Span) -> Result<Self, CompileError> {
        if raw > crate::asm_generation::compiler_constants::TWENTY_FOUR_BITS {
            Err(CompileError::Immediate24TooLarge {
                val: raw,
                span: err_msg_span,
            })
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().unwrap_or_else(|_| panic!("{}", &msg.into())),
        }
    }
}
impl fmt::Display for VirtualImmediate24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}
