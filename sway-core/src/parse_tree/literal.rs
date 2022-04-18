use crate::{
    build_config::BuildConfig, error::*, parser::Rule, type_engine::IntegerBits,
    types::ResolvedType, CompileError, TypeInfo,
};

use sway_types::span;

use pest::iterators::Pair;
use pest::Span;

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
    pub(crate) fn parse_from_pair(
        lit: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<(Self, span::Span)> {
        let path = config.map(|c| c.path());
        let lit_inner = lit.into_inner().next().unwrap();
        let (parsed, span): (Result<Literal, CompileError>, _) = match lit_inner.as_rule() {
            Rule::basic_integer => {
                let span = span::Span::from_pest(lit_inner.as_span(), path);
                (
                    lit_inner
                        .as_str()
                        .trim()
                        .replace('_', "")
                        .parse()
                        .map(Literal::Numeric)
                        .map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                                span.clone(),
                            )
                        }),
                    span,
                )
            }
            Rule::typed_integer => {
                let mut int_inner = lit_inner.into_inner().next().unwrap();
                let rule = int_inner.as_rule();
                if int_inner.as_rule() != Rule::basic_integer {
                    int_inner = int_inner.into_inner().next().unwrap()
                }
                let span = span::Span::from_pest(int_inner.as_span(), path);
                (
                    match rule {
                        Rule::u8_integer => int_inner
                            .as_str()
                            .trim()
                            .replace('_', "")
                            .parse()
                            .map(Literal::U8)
                            .map_err(|e| {
                                Literal::handle_parse_int_error(
                                    e,
                                    TypeInfo::UnsignedInteger(IntegerBits::Eight),
                                    span.clone(),
                                )
                            }),
                        Rule::u16_integer => int_inner
                            .as_str()
                            .trim()
                            .replace('_', "")
                            .parse()
                            .map(Literal::U16)
                            .map_err(|e| {
                                Literal::handle_parse_int_error(
                                    e,
                                    TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                                    span.clone(),
                                )
                            }),
                        Rule::u32_integer => int_inner
                            .as_str()
                            .trim()
                            .replace('_', "")
                            .parse()
                            .map(Literal::U32)
                            .map_err(|e| {
                                Literal::handle_parse_int_error(
                                    e,
                                    TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                                    span.clone(),
                                )
                            }),
                        Rule::u64_integer => int_inner
                            .as_str()
                            .trim()
                            .replace('_', "")
                            .parse()
                            .map(Literal::U64)
                            .map_err(|e| {
                                Literal::handle_parse_int_error(
                                    e,
                                    TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                                    span.clone(),
                                )
                            }),
                        _ => unreachable!(),
                    },
                    span,
                )
            }
            Rule::string => {
                // remove opening and closing quotes
                let lit_span = lit_inner.as_span();
                let lit = span::Span::new(
                    lit_span.input().clone(),
                    lit_span.start() + 1,
                    lit_span.end() - 1,
                    path.clone(),
                )
                .unwrap();
                let span = span::Span::from_pest(lit_span, path);
                (Ok(Literal::String(lit)), span)
            }
            Rule::byte => {
                let inner_byte = lit_inner.into_inner().next().unwrap();
                let span = span::Span::from_pest(inner_byte.as_span(), path);
                (
                    match inner_byte.as_rule() {
                        Rule::binary_byte => parse_binary_from_pair(inner_byte, config),
                        Rule::hex_byte => parse_hex_from_pair(inner_byte, config),
                        _ => unreachable!(),
                    },
                    span,
                )
            }
            Rule::boolean => {
                let span = span::Span::from_pest(lit_inner.as_span(), path);
                (
                    Ok(match lit_inner.as_str() {
                        "true" => Literal::Boolean(true),
                        "false" => Literal::Boolean(false),
                        _ => unreachable!(),
                    }),
                    span,
                )
            }
            a => {
                eprintln!(
                    "not yet able to parse literal rule {:?} ({:?})",
                    a,
                    lit_inner.as_str()
                );
                (
                    Err(CompileError::UnimplementedRule(
                        a,
                        span::Span::from_pest(lit_inner.as_span(), path.clone()),
                    )),
                    span::Span::from_pest(lit_inner.as_span(), path),
                )
            }
        };

        match parsed {
            Ok(lit) => ok((lit, span), Vec::new(), Vec::new()),
            Err(compile_err) => err(Vec::new(), vec![compile_err]),
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

fn parse_hex_from_pair(
    pair: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> Result<Literal, CompileError> {
    let path = config.map(|c| c.path());
    let hex = &pair.as_str()[2..]
        .chars()
        .filter(|x| *x != '_')
        .collect::<String>();

    Ok(match hex.len() {
        2 => Literal::Byte(u8::from_str_radix(hex, 16).map_err(|_| {
            CompileError::Internal(
                "Attempted to parse hex string from invalid hex",
                span::Span::from_pest(pair.as_span(), path.clone()),
            )
        })?),
        64 => {
            let vec_nums: Vec<u8> = hex
                .chars()
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|two_hex_digits| -> Result<u8, CompileError> {
                    let mut str_buf = String::new();
                    two_hex_digits.iter().for_each(|x| str_buf.push(*x));
                    u8::from_str_radix(&str_buf, 16).map_err(|_| {
                        CompileError::Internal(
                            "Attempted to parse individual byte from invalid hex string.",
                            span::Span::from_pest(pair.as_span(), path.clone()),
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|_| {
                CompileError::Internal(
                    "Attempted to parse bytes32 from hex literal of incorrect length. ",
                    span::Span::from_pest(pair.as_span(), path.clone()),
                )
            })?;
            Literal::B256(arr)
        }
        a => {
            return Err(CompileError::InvalidByteLiteralLength {
                span: span::Span::from_pest(pair.as_span(), path),
                byte_length: a,
            })
        }
    })
}

fn parse_binary_from_pair(
    pair: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> Result<Literal, CompileError> {
    let path = config.map(|c| c.path());
    let bin = &pair.as_str()[2..]
        .chars()
        .filter(|x| *x != '_')
        .collect::<String>();

    Ok(match bin.len() {
        8 => Literal::Byte(u8::from_str_radix(bin, 2).map_err(|_| {
            CompileError::Internal(
                "Attempted to parse bin string from invalid bin string.",
                span::Span::from_pest(pair.as_span(), path.clone()),
            )
        })?),
        256 => {
            let vec_nums: Vec<u8> = bin
                .chars()
                .collect::<Vec<_>>()
                .chunks(8)
                .map(|eight_bin_digits| -> Result<u8, CompileError> {
                    let mut str_buf = String::new();
                    eight_bin_digits.iter().for_each(|x| str_buf.push(*x));
                    u8::from_str_radix(&str_buf, 2).map_err(|_| {
                        CompileError::Internal(
                            "Attempted to parse individual byte from invalid bin.",
                            span::Span::from_pest(pair.as_span(), path.clone()),
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|_| {
                CompileError::Internal(
                    "Attempted to parse bytes32 from bin literal of incorrect length. ",
                    span::Span::from_pest(pair.as_span(), path.clone()),
                )
            })?;
            Literal::B256(arr)
        }
        a => {
            return Err(CompileError::InvalidByteLiteralLength {
                span: span::Span::from_pest(pair.as_span(), path),
                byte_length: a,
            })
        }
    })
}

#[allow(clippy::wildcard_in_or_patterns)]
pub(crate) fn handle_parse_int_error(
    e: ParseIntError,
    ty: TypeInfo,
    span: Span,
    path: Option<Arc<PathBuf>>,
) -> CompileError {
    match e.kind() {
        IntErrorKind::PosOverflow => CompileError::IntegerTooLarge {
            ty: ty.friendly_type_str(),
            span: span::Span::from_pest(span, path),
        },
        IntErrorKind::NegOverflow => CompileError::IntegerTooSmall {
            ty: ty.friendly_type_str(),
            span: span::Span::from_pest(span, path),
        },
        IntErrorKind::InvalidDigit => CompileError::IntegerContainsInvalidDigit {
            ty: ty.friendly_type_str(),
            span: span::Span::from_pest(span, path),
        },
        IntErrorKind::Zero | IntErrorKind::Empty | _ => CompileError::Internal(
            "Called incorrect internal sway-core on literal type.",
            span::Span::from_pest(span, path),
        ),
    }
}
