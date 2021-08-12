use crate::error::*;
use crate::parser::Rule;
use crate::types::{IntegerBits, ResolvedType};
use crate::CompileError;
use pest::iterators::Pair;
use pest::Span;
use std::convert::TryInto;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Literal<'sc> {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    String(&'sc str),
    Boolean(bool),
    Byte(u8),
    Byte32([u8; 32]),
}

impl<'sc> Literal<'sc> {
    #[allow(dead_code)]
    pub(crate) fn as_type(&self) -> ResolvedType<'sc> {
        use Literal::*;
        match self {
            U8(_) => ResolvedType::UnsignedInteger(IntegerBits::Eight),
            U16(_) => ResolvedType::UnsignedInteger(IntegerBits::Sixteen),
            U32(_) => ResolvedType::UnsignedInteger(IntegerBits::ThirtyTwo),
            U64(_) => ResolvedType::UnsignedInteger(IntegerBits::SixtyFour),
            String(inner) => ResolvedType::Str(inner.len() as u64),
            Boolean(_) => ResolvedType::Boolean,
            Byte(_) => ResolvedType::Byte,
            Byte32(_) => ResolvedType::Byte32,
        }
    }
    pub(crate) fn parse_from_pair(lit: Pair<'sc, Rule>) -> CompileResult<'sc, (Self, Span<'sc>)> {
        let lit_inner = lit.into_inner().next().unwrap();
        let (parsed, span): (Result<Literal, CompileError>, _) = match lit_inner.as_rule() {
            Rule::integer => {
                let mut int_inner = lit_inner.into_inner().next().unwrap();
                let rule = int_inner.as_rule();
                if int_inner.as_rule() != Rule::basic_integer {
                    int_inner = int_inner.into_inner().next().unwrap()
                }
                let span = int_inner.as_span();
                (
                    match rule {
                        Rule::u8_integer => int_inner
                            .as_str()
                            .trim()
                            .replace("_", "")
                            .parse()
                            .map(Literal::U8)
                            .map_err(|_| {
                                CompileError::Internal(
                                    "Called incorrect internal core_lang on literal type.",
                                    int_inner.as_span(),
                                )
                            }),
                        Rule::u16_integer => int_inner
                            .as_str()
                            .trim()
                            .replace("_", "")
                            .parse()
                            .map(Literal::U16)
                            .map_err(|_| {
                                CompileError::Internal(
                                    "Called incorrect internal core_lang on literal type.",
                                    int_inner.as_span(),
                                )
                            }),
                        Rule::u32_integer => int_inner
                            .as_str()
                            .trim()
                            .replace("_", "")
                            .parse()
                            .map(Literal::U32)
                            .map_err(|_| {
                                CompileError::Internal(
                                    "Called incorrect internal core_lang on literal type.",
                                    int_inner.as_span(),
                                )
                            }),
                        Rule::u64_integer => int_inner
                            .as_str()
                            .trim()
                            .replace("_", "")
                            .parse()
                            .map(Literal::U64)
                            .map_err(|_| {
                                CompileError::Internal(
                                    "Called incorrect internal core_lang on literal type.",
                                    int_inner.as_span(),
                                )
                            }),
                        _ => unreachable!(),
                    },
                    span,
                )
            }
            Rule::string => {
                // remove opening and closing quotes
                let lit_str = lit_inner.as_str();
                let span = lit_inner.as_span();
                (Ok(Literal::String(&lit_str[1..lit_str.len() - 1])), span)
            }
            Rule::byte => {
                let inner_byte = lit_inner.into_inner().next().unwrap();
                let span = inner_byte.as_span();
                (
                    match inner_byte.as_rule() {
                        Rule::binary_byte => parse_binary_from_pair(inner_byte),
                        Rule::hex_byte => parse_hex_from_pair(inner_byte),
                        _ => unreachable!(),
                    },
                    span,
                )
            }
            Rule::boolean => {
                let span = lit_inner.as_span();
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
                    Err(CompileError::UnimplementedRule(a, lit_inner.as_span())),
                    lit_inner.as_span(),
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
            Boolean(b) => {
                let bytes = (if *b { 1u64 } else { 0u64 }).to_be_bytes();
                vec![0, 0, 0, 0, 0, 0, 0, bytes[0]]
            }
            // assume utf8 for now
            String(st) => st.to_string().into_bytes(),
            Byte(b) => vec![0, 0, 0, 0, 0, 0, 0, b.to_be_bytes()[0]],
            Byte32(b) => b.to_vec(),
        }
    }
}

fn parse_hex_from_pair<'sc>(pair: Pair<'sc, Rule>) -> Result<Literal<'sc>, CompileError<'sc>> {
    let hex = &pair.as_str()[2..]
        .chars()
        .filter(|x| *x != '_')
        .collect::<String>();

    Ok(match hex.len() {
        2 => Literal::Byte(u8::from_str_radix(hex, 16).map_err(|_| {
            CompileError::Internal(
                "Attempted to parse hex string from invalid hex",
                pair.as_span(),
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
                    Ok(u8::from_str_radix(&str_buf, 16).map_err(|_| {
                        CompileError::Internal(
                            "Attempted to parse individual byte from invalid hex string.",
                            pair.as_span(),
                        )
                    })?)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|_| {
                CompileError::Internal(
                    "Attempted to parse bytes32 from hex literal of incorrect length. ",
                    pair.as_span(),
                )
            })?;
            Literal::Byte32(arr)
        }
        a => {
            return Err(CompileError::InvalidByteLiteralLength {
                span: pair.as_span(),
                byte_length: a,
            })
        }
    })
}

fn parse_binary_from_pair<'sc>(pair: Pair<'sc, Rule>) -> Result<Literal<'sc>, CompileError<'sc>> {
    let bin = &pair.as_str()[2..]
        .chars()
        .filter(|x| *x != '_')
        .collect::<String>();

    Ok(match bin.len() {
        8 => Literal::Byte(u8::from_str_radix(bin, 2).map_err(|_| {
            CompileError::Internal(
                "Attempted to parse bin string from invalid bin string.",
                pair.as_span(),
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
                    Ok(u8::from_str_radix(&str_buf, 2).map_err(|_| {
                        CompileError::Internal(
                            "Attempted to parse individual byte from invalid bin.",
                            pair.as_span(),
                        )
                    })?)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|_| {
                CompileError::Internal(
                    "Attempted to parse bytes32 from bin literal of incorrect length. ",
                    pair.as_span(),
                )
            })?;
            Literal::Byte32(arr)
        }
        a => {
            return Err(CompileError::InvalidByteLiteralLength {
                span: pair.as_span(),
                byte_length: a,
            })
        }
    })
}
