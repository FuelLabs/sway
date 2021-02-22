use crate::parser::Rule;
use crate::ParseError;
use pest::iterators::Pair;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub(crate) enum Literal<'sc> {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    String(&'sc str),
    Boolean(bool),
    Byte(u8),
    Byte32([u8; 32]),
}

impl<'sc> Literal<'sc> {
    pub(crate) fn parse_from_pair(lit: Pair<'sc, Rule>) -> Result<Self, ParseError<'sc>> {
        let lit_inner = lit.into_inner().next().unwrap();
        let parsed = match lit_inner.as_rule() {
            Rule::integer => {
                let int_inner = lit_inner.into_inner().next().unwrap();
                match int_inner.as_rule() {
                    Rule::u8_integer => {
                        Literal::U8(int_inner.as_str().trim().parse().map_err(|e| {
                            ParseError::Internal(
                                "Called incorrect internal parser on literal type.",
                                int_inner.into_span(),
                            )
                        })?)
                    }
                    Rule::u16_integer => {
                        Literal::U16(int_inner.as_str().trim().parse().map_err(|e| {
                            ParseError::Internal(
                                "Called incorrect internal parser on literal type.",
                                int_inner.into_span(),
                            )
                        })?)
                    }
                    Rule::u32_integer => {
                        Literal::U32(int_inner.as_str().trim().parse().map_err(|e| {
                            ParseError::Internal(
                                "Called incorrect internal parser on literal type.",
                                int_inner.into_span(),
                            )
                        })?)
                    }
                    Rule::u64_integer => {
                        Literal::U64(int_inner.as_str().trim().parse().map_err(|e| {
                            ParseError::Internal(
                                "Called incorrect internal parser on literal type.",
                                int_inner.into_span(),
                            )
                        })?)
                    }
                    Rule::u128_integer => {
                        Literal::U128(int_inner.as_str().trim().parse().map_err(|e| {
                            ParseError::Internal(
                                "Called incorrect internal parser on literal type.",
                                int_inner.into_span(),
                            )
                        })?)
                    }
                    _ => unreachable!(),
                }
            }
            Rule::string => {
                // remove opening and closing quotes
                let lit_str = lit_inner.as_str();
                Literal::String(&lit_str[1..lit_str.len() - 1])
            }
            Rule::byte => {
                let inner_byte = lit_inner.into_inner().next().unwrap();
                match inner_byte.as_rule() {
                    Rule::binary_byte => parse_binary_from_pair(inner_byte)?,
                    Rule::hex_byte => parse_hex_from_pair(inner_byte)?,
                    _ => unreachable!(),
                }
            }
            Rule::boolean => match lit_inner.as_str() {
                "true" => Literal::Boolean(true),
                "false" => Literal::Boolean(false),
                _ => unreachable!(),
            },
            a => {
                eprintln!(
                    "not yet able to parse literal rule {:?} ({:?})",
                    a,
                    lit_inner.as_str()
                );
                return Err(ParseError::Unimplemented(a, lit_inner.as_span()));
            }
        };

        Ok(parsed)
    }
}

fn parse_hex_from_pair<'sc>(pair: Pair<'sc, Rule>) -> Result<Literal<'sc>, ParseError<'sc>> {
    let hex = &pair.as_str()[2..];
    Ok(match hex.len() {
        2 => Literal::Byte(u8::from_str_radix(hex, 16).map_err(|e| {
            ParseError::Internal(
                "Attempted to parse hex string from invalid hex",
                pair.as_span(),
            )
        })?),
        64 => {
            let vec_nums: Vec<u8> = hex
                .chars()
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|two_hex_digits| -> Result<u8, ParseError> {
                    let mut str_buf = String::new();
                    two_hex_digits.iter().for_each(|x| str_buf.push(*x));
                    Ok(u8::from_str_radix(&str_buf, 16).map_err(|_| {
                        ParseError::Internal(
                            "Attempted to parse individual byte from invalid hex string.",
                            pair.as_span(),
                        )
                    })?)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|e| {
                ParseError::Internal(
                    "Attempted to parse bytes32 from hex literal of incorrect length. ",
                    pair.as_span(),
                )
            })?;
            Literal::Byte32(arr)
        }
        a => {
            return Err(ParseError::InvalidByteLiteralLength {
                span: pair.as_span(),
                byte_length: a,
            })
        }
    })
}

fn parse_binary_from_pair<'sc>(pair: Pair<'sc, Rule>) -> Result<Literal<'sc>, ParseError<'sc>> {
    let bin = &pair.as_str()[2..];

    Ok(match bin.len() {
        8 => Literal::Byte(u8::from_str_radix(bin, 2).map_err(|e| {
            ParseError::Internal(
                "Attempted to parse bin string from invalid bin string.",
                pair.as_span(),
            )
        })?),
        256 => {
            let vec_nums: Vec<u8> = bin
                .chars()
                .collect::<Vec<_>>()
                .chunks(8)
                .map(|eight_bin_digits| -> Result<u8, ParseError> {
                    let mut str_buf = String::new();
                    eight_bin_digits.iter().for_each(|x| str_buf.push(*x));
                    Ok(u8::from_str_radix(&str_buf, 2).map_err(|_| {
                        ParseError::Internal(
                            "Attempted to parse individual byte from invalid bin.",
                            pair.as_span(),
                        )
                    })?)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|e| {
                ParseError::Internal(
                    "Attempted to parse bytes32 from bin literal of incorrect length. ",
                    pair.as_span(),
                )
            })?;
            Literal::Byte32(arr)
        }
        a => {
            return Err(ParseError::InvalidByteLiteralLength {
                span: pair.as_span(),
                byte_length: a,
            })
        }
    })
}
