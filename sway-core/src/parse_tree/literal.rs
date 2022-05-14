use crate::{
    build_config::BuildConfig, error::*, span::Span, type_engine::IntegerBits, types::ResolvedType,
    CompileError, TypeInfo,
};

use sway_types::span;

use std::{
    convert::TryInto,
    hash::{Hash, Hasher},
    num::{IntErrorKind, ParseIntError},
    path::PathBuf,
    sync::Arc,
};

#[derive(Debug, Clone, Eq)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    String(span::Span),
    Numeric(u64),
    Boolean(bool),
    Byte(u8),
    B256([u8; 32]),
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
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
            Byte(x) => {
                state.write_u8(8);
                x.hash(state);
            }
            B256(x) => {
                state.write_u8(9);
                x.hash(state);
            }
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::U8(l0), Self::U8(r0)) => l0 == r0,
            (Self::U16(l0), Self::U16(r0)) => l0 == r0,
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => *l0.as_str() == *r0.as_str(),
            (Self::Numeric(l0), Self::Numeric(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Byte(l0), Self::Byte(r0)) => l0 == r0,
            (Self::B256(l0), Self::B256(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Literal {
    #[allow(dead_code)]
    pub(crate) fn as_type(&self) -> ResolvedType {
        use Literal::*;
        match self {
            U8(_) => ResolvedType::UnsignedInteger(IntegerBits::Eight),
            U16(_) => ResolvedType::UnsignedInteger(IntegerBits::Sixteen),
            U32(_) => ResolvedType::UnsignedInteger(IntegerBits::ThirtyTwo),
            U64(_) => ResolvedType::UnsignedInteger(IntegerBits::SixtyFour),
            Numeric(_) => ResolvedType::UnsignedInteger(IntegerBits::SixtyFour),
            String(inner) => ResolvedType::Str(inner.as_str().len() as u64),
            Boolean(_) => ResolvedType::Boolean,
            Byte(_) => ResolvedType::Byte,
            B256(_) => ResolvedType::B256,
        }
    }
    /// Converts a literal to a big-endian representation. This is padded to words.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        use Literal::*;
        match self {
            U8(val) => vec![0, 0, 0, 0, 0, 0, 0, val.to_be_bytes()[0]],
            U16(val) => {
                let bytes = val.to_be_bytes();
                vec![0, 0, 0, 0, 0, 0, bytes[0], bytes[1]]
            }
            U32(val) => {
                let bytes = val.to_be_bytes();
                vec![0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]]
            }
            U64(val) => val.to_be_bytes().to_vec(),
            Numeric(val) => val.to_be_bytes().to_vec(),
            Boolean(b) => {
                vec![
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    if *b { 0b00000001 } else { 0b00000000 },
                ]
            }
            // assume utf8 for now
            String(st) => {
                let mut buf = st.as_str().to_string().into_bytes();
                // pad to word alignment
                while buf.len() % 8 != 0 {
                    buf.push(0);
                }
                buf
            }
            Byte(b) => vec![0, 0, 0, 0, 0, 0, 0, b.to_be_bytes()[0]],
            B256(b) => b.to_vec(),
        }
    }

    /// Used when creating a pointer literal value, typically during code generation for
    /// values that wouldn't fit in a register.
    pub(crate) fn new_pointer_literal(offset_bytes: u64) -> Literal {
        Literal::U64(offset_bytes)
    }

    #[allow(clippy::wildcard_in_or_patterns)]
    pub(crate) fn handle_parse_int_error(
        e: ParseIntError,
        ty: TypeInfo,
        span: sway_types::Span,
    ) -> CompileError {
        match e.kind() {
            IntErrorKind::PosOverflow => CompileError::IntegerTooLarge {
                ty: ty.friendly_type_str(),
                span,
            },
            IntErrorKind::NegOverflow => CompileError::IntegerTooSmall {
                ty: ty.friendly_type_str(),
                span,
            },
            IntErrorKind::InvalidDigit => CompileError::IntegerContainsInvalidDigit {
                ty: ty.friendly_type_str(),
                span,
            },
            IntErrorKind::Zero | IntErrorKind::Empty | _ => {
                CompileError::Internal("Called incorrect internal sway-core on literal type.", span)
            }
        }
    }
}
