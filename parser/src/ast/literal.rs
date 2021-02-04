use crate::parser::Rule;
use crate::CompileError;
use pest::iterators::Pair;
use std::convert::TryInto;

#[derive(Debug)]
pub(crate) enum Literal<'sc> {
    Integer(i64),
    String(&'sc str),
    Boolean(bool),
    Byte(u8),
    Byte32([u8; 32]),
}

impl<'sc> Literal<'sc> {
    pub(crate) fn parse_from_pair(lit: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let lit_inner = lit.into_inner().next().unwrap();
        let parsed = match lit_inner.as_rule() {
            Rule::integer => Literal::Integer(lit_inner.as_str().parse().map_err(|e| {
                CompileError::Internal(
                    "Called incorrect internal parser on literal type.",
                    lit_inner.into_span(),
                )
            })?),
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
            a => {
                eprintln!(
                    "not yet able to parse literal rule {:?} ({:?})",
                    a,
                    lit_inner.as_str()
                );
                return Err(CompileError::Unimplemented(a, lit_inner.as_span()));
            }
        };

        Ok(parsed)
    }
}

fn parse_hex_from_pair<'sc>(pair: Pair<'sc, Rule>) -> Result<Literal<'sc>, CompileError<'sc>> {
    let hex = &pair.as_str()[2..];
    Ok(match hex.len() {
        2 => Literal::Byte(u8::from_str_radix(hex, 16).map_err(|e| {
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
                    Ok(u8::from_str_radix(&str_buf, 16).map_err(|_| CompileError::Internal(
                        "Attempted to parse individual byte from invalid hex string.", pair.as_span()    
                    ))?)
                })
                .collect::<Result<Vec<_>, _>>()?;
                let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|e| CompileError::Internal(
                    "Attempted to parse bytes32 from hex literal of incorrect length. ",
                    pair.as_span(),

                        ))?;
            Literal::Byte32(arr)
            }
        a => return Err(CompileError::InvalidByteLiteralLength {
            span: pair.as_span(),
            byte_length: a
        })
    })
}

fn parse_binary_from_pair<'sc>(pair: Pair<'sc, Rule>) -> Result<Literal<'sc>, CompileError<'sc>> {
    let bin = &pair.as_str()[2..];

    Ok(match bin.len() {
        8 => Literal::Byte(u8::from_str_radix(bin, 2).map_err(|e| {
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
                    Ok(u8::from_str_radix(&str_buf, 2).map_err(|_| CompileError::Internal(
                        "Attempted to parse individual byte from invalid bin.", pair.as_span()    
                    ))?)
                })
                .collect::<Result<Vec<_>, _>>()?;
                let arr: [u8; 32] = vec_nums.as_slice().try_into().map_err(|e| CompileError::Internal(
                    "Attempted to parse bytes32 from bin literal of incorrect length. ",
                    pair.as_span(),

                        ))?;
            Literal::Byte32(arr)
            }
        a => return Err(CompileError::InvalidByteLiteralLength {
            span: pair.as_span(),
            byte_length: a
        })
    })
}
