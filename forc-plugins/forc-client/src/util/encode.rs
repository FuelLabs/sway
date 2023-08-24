use anyhow::Context;
use fuel_abi_types::abi::full_program::FullTypeApplication;
use std::str::FromStr;
use sway_types::u256::U256;

/// A wrapper around fuels_core::types::Token, which enables serde de/serialization.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Token(pub(crate) fuels_core::types::Token);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Unit,
    U8,
    U16,
    U32,
    U64,
    U256,
    Bool,
}

impl TryFrom<&FullTypeApplication> for Type {
    type Error = anyhow::Error;

    fn try_from(value: &FullTypeApplication) -> Result<Self, Self::Error> {
        let type_field_string = &value.type_decl.type_field;
        Type::from_str(type_field_string)
    }
}

impl Token {
    /// Generate a new token using provided type information and the value for the argument.
    ///
    /// Generates an error if there is a mismatch between the type information and the provided
    /// value for that type.
    #[allow(dead_code)]
    pub(crate) fn from_type_and_value(arg_type: &Type, value: &str) -> anyhow::Result<Self> {
        match arg_type {
            Type::Unit => Ok(Token(fuels_core::types::Token::Unit)),
            Type::U8 => {
                let u8_val = value.parse::<u8>()?;
                Ok(Token(fuels_core::types::Token::U8(u8_val)))
            }
            Type::U16 => {
                let u16_val = value.parse::<u16>()?;
                Ok(Token(fuels_core::types::Token::U16(u16_val)))
            }
            Type::U32 => {
                let u32_val = value.parse::<u32>()?;
                Ok(Token(fuels_core::types::Token::U32(u32_val)))
            }
            Type::U64 => {
                let u64_val = value.parse::<u64>()?;
                Ok(Token(fuels_core::types::Token::U64(u64_val)))
            }
            Type::U256 => {
                let v = value.parse::<U256>().context("u256 literal out of range")?;
                let bytes = v.to_be_bytes();
                Ok(Token(fuels_core::types::Token::U256(bytes.into())))
            }
            Type::Bool => {
                let bool_val = value.parse::<bool>()?;
                Ok(Token(fuels_core::types::Token::Bool(bool_val)))
            }
        }
    }
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "()" => Ok(Type::Unit),
            "u8" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "u256" => Ok(Type::U256),
            "bool" => Ok(Type::Bool),
            other => anyhow::bail!("{other} type is not supported."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_success() {
        let u8_token = Token::from_type_and_value(&Type::U8, "1").unwrap();
        let u16_token = Token::from_type_and_value(&Type::U16, "1").unwrap();
        let u32_token = Token::from_type_and_value(&Type::U32, "1").unwrap();
        let u64_token = Token::from_type_and_value(&Type::U64, "1").unwrap();
        let bool_token = Token::from_type_and_value(&Type::Bool, "true").unwrap();

        let generated_tokens = [u8_token, u16_token, u32_token, u64_token, bool_token];
        let expected_tokens = [
            Token(fuels_core::types::Token::U8(1)),
            Token(fuels_core::types::Token::U16(1)),
            Token(fuels_core::types::Token::U32(1)),
            Token(fuels_core::types::Token::U64(1)),
            Token(fuels_core::types::Token::Bool(true)),
        ];

        assert_eq!(generated_tokens, expected_tokens)
    }

    #[test]
    #[should_panic]
    fn test_token_generation_fail_type_mismatch() {
        Token::from_type_and_value(&Type::U8, "false").unwrap();
    }

    #[test]
    fn test_type_generation_success() {
        let possible_type_list = ["()", "u8", "u16", "u32", "u64", "u256", "bool"];
        let types = possible_type_list
            .iter()
            .map(|type_str| Type::from_str(type_str))
            .collect::<anyhow::Result<Vec<_>>>()
            .unwrap();

        let expected_types = vec![
            Type::Unit,
            Type::U8,
            Type::U16,
            Type::U32,
            Type::U64,
            Type::U256,
            Type::Bool,
        ];
        assert_eq!(types, expected_types)
    }

    #[test]
    #[should_panic(expected = "u2 type is not supported.")]
    fn test_type_generation_fail_invalid_type() {
        let invalid_type_str = "u2";
        Type::from_str(invalid_type_str).unwrap();
    }
}
