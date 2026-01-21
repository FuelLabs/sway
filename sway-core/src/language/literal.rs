use crate::{type_system::*, Engines};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
    num::{IntErrorKind, ParseIntError},
};
use sway_error::error::CompileError;
use sway_types::{integer_bits::IntegerBits, span, u256::U256};

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U256(U256),
    String(span::Span),
    Numeric(u64),
    Boolean(bool),
    B256([u8; 32]),
    Binary(Vec<u8>),
}

impl Literal {
    pub fn cast_value_to_u64(&self) -> Option<u64> {
        match self {
            Literal::U8(v) => Some(*v as u64),
            Literal::U16(v) => Some(*v as u64),
            Literal::U32(v) => Some(*v as u64),
            Literal::U64(v) => Some(*v),
            Literal::Numeric(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns `true` if the runtime memory representation of a
    /// type instance memcmp-equal to a constant represented by this [Literal]
    /// would always be all zeros.
    pub fn is_runtime_zeroed(&self) -> bool {
        match self {
            Literal::U8(v) => *v == 0,
            Literal::U16(v) => *v == 0,
            Literal::U32(v) => *v == 0,
            Literal::U64(v) => *v == 0,
            Literal::U256(v) => v.is_zero(),
            // String is a string slice resulting in a fat pointer, so not zero.
            Literal::String(_) => false,
            Literal::Numeric(v) => *v == 0,
            Literal::Boolean(v) => !(*v),
            Literal::B256(v) => v.iter().all(|b| *b == 0),
            Literal::Binary(v) => v.iter().all(|b| *b == 0),
        }
    }
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Literal::*;
        match self {
            U8(x) => {
                state.write_u8(1);
                x.hash(state);
            }
            U16(x) => {
                state.write_u8(2);
                x.hash(state);
            }
            U32(x) => {
                state.write_u8(3);
                x.hash(state);
            }
            U64(x) => {
                state.write_u8(4);
                x.hash(state);
            }
            U256(x) => {
                state.write_u8(4);
                x.hash(state);
            }
            Numeric(x) => {
                state.write_u8(5);
                x.hash(state);
            }
            String(inner) => {
                state.write_u8(6);
                inner.as_str().hash(state);
            }
            Boolean(x) => {
                state.write_u8(7);
                x.hash(state);
            }
            B256(x) => {
                state.write_u8(8);
                x.hash(state);
            }
            Binary(x) => {
                state.write_u8(9);
                x.hash(state);
            }
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::U8(l0), Self::U8(r0)) => l0 == r0,
            (Self::U16(l0), Self::U16(r0)) => l0 == r0,
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::U256(l0), Self::U256(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => *l0.as_str() == *r0.as_str(),
            (Self::Numeric(l0), Self::Numeric(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::B256(l0), Self::B256(r0)) => l0 == r0,
            (Self::Binary(l0), Self::Binary(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Literal::U8(content) => content.to_string(),
            Literal::U16(content) => content.to_string(),
            Literal::U32(content) => content.to_string(),
            Literal::U64(content) => content.to_string(),
            Literal::U256(content) => content.to_string(),
            Literal::Numeric(content) => content.to_string(),
            Literal::String(content) => content.as_str().to_string(),
            Literal::Boolean(content) => content.to_string(),
            Literal::B256(content) => content
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            Literal::Binary(content) => content
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        };
        write!(f, "{s}")
    }
}

impl Literal {
    #[allow(clippy::wildcard_in_or_patterns)]
    pub(crate) fn handle_parse_int_error(
        engines: &Engines,
        e: ParseIntError,
        ty: TypeInfo,
        span: sway_types::Span,
    ) -> CompileError {
        match e.kind() {
            IntErrorKind::PosOverflow => CompileError::IntegerTooLarge {
                ty: engines.help_out(ty).to_string(),
                span,
            },
            IntErrorKind::NegOverflow => CompileError::IntegerTooSmall {
                ty: engines.help_out(ty).to_string(),
                span,
            },
            IntErrorKind::InvalidDigit => CompileError::IntegerContainsInvalidDigit {
                ty: engines.help_out(ty).to_string(),
                span,
            },
            IntErrorKind::Zero | IntErrorKind::Empty | _ => {
                CompileError::Internal("Called incorrect internal sway-core on literal type.", span)
            }
        }
    }

    pub(crate) fn to_typeinfo(&self) -> TypeInfo {
        match self {
            Literal::String(_) => TypeInfo::StringSlice,
            Literal::Numeric(_) => TypeInfo::Numeric,
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::U256(_) => TypeInfo::UnsignedInteger(IntegerBits::V256),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::B256(_) => TypeInfo::B256,
            Literal::Binary(_) => TypeInfo::RawUntypedSlice,
        }
    }
}
