use anyhow::Context;
use fuel_abi_types::abi::full_program::FullTypeApplication;
use fuels::types::{Bits256, StaticStringToken, U256};
use std::str::FromStr;

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
    U128,
    U256,
    B256,
    Bool,
    String,
    StringArray(usize),
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
                let v = value.parse::<U256>().context("Invalid value for U256")?;
                Ok(Token(fuels_core::types::Token::U256(v)))
            }
            Type::Bool => {
                let bool_val = value.parse::<bool>()?;
                Ok(Token(fuels_core::types::Token::Bool(bool_val)))
            }
            Type::U128 => {
                let u128_val = value.parse::<u128>()?;
                Ok(Token(fuels_core::types::Token::U128(u128_val)))
            }
            Type::B256 => {
                let bits256 = Bits256::from_hex_str(value)?;
                Ok(Token(fuels_core::types::Token::B256(bits256.0)))
            }
            Type::String => {
                let s = value.to_string();
                Ok(Token(fuels_core::types::Token::String(s)))
            }
            Type::StringArray(len) => {
                let s = StaticStringToken::new(value.to_string(), Some(*len));
                Ok(Token(fuels_core::types::Token::StringArray(s)))
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
            "u128" => Ok(Type::U128),
            "u256" => Ok(Type::U256),
            "bool" => Ok(Type::Bool),
            "b256" => Ok(Type::B256),
            "String" => Ok(Type::String),
            other => {
                // Use a regular expression to check for string slice syntax
                let re = forc_util::Regex::new(r"^str\[(\d+)\]$").expect("Invalid regex pattern");
                if let Some(captures) = re.captures(other) {
                    // Extract the number inside the brackets
                    let len = captures
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Invalid string slice length"))?
                        .as_str()
                        .parse::<usize>()
                        .map_err(|_| anyhow::anyhow!("Invalid number for string slice length"))?;
                    Ok(Type::StringArray(len))
                } else {
                    anyhow::bail!("{other} type is not supported.")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuel_abi_types::abi::full_program::{FullTypeApplication, FullTypeDeclaration};
    use fuels::types::StaticStringToken;

    #[test]
    fn test_token_generation_success() {
        let u8_token = Token::from_type_and_value(&Type::U8, "1").unwrap();
        let u16_token = Token::from_type_and_value(&Type::U16, "1").unwrap();
        let u32_token = Token::from_type_and_value(&Type::U32, "1").unwrap();
        let u64_token = Token::from_type_and_value(&Type::U64, "1").unwrap();
        let u128_token = Token::from_type_and_value(&Type::U128, "1").unwrap();
        let u256_token = Token::from_type_and_value(&Type::U256, "1").unwrap();
        let b256_token = Token::from_type_and_value(
            &Type::B256,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let bool_token = Token::from_type_and_value(&Type::Bool, "true").unwrap();
        let string_token = Token::from_type_and_value(&Type::String, "Hello, World!").unwrap();
        let string_array_token =
            Token::from_type_and_value(&Type::StringArray(5), "Hello").unwrap();

        let generated_tokens = [
            u8_token,
            u16_token,
            u32_token,
            u64_token,
            u128_token,
            u256_token,
            b256_token,
            bool_token,
            string_token,
            string_array_token,
        ];
        let expected_tokens = [
            Token(fuels_core::types::Token::U8(1)),
            Token(fuels_core::types::Token::U16(1)),
            Token(fuels_core::types::Token::U32(1)),
            Token(fuels_core::types::Token::U64(1)),
            Token(fuels_core::types::Token::U128(1)),
            Token(fuels_core::types::Token::U256(fuels::types::U256([
                1, 0, 0, 0,
            ]))),
            Token(fuels_core::types::Token::B256([0; 32])),
            Token(fuels_core::types::Token::Bool(true)),
            Token(fuels_core::types::Token::String(
                "Hello, World!".to_string(),
            )),
            Token(fuels_core::types::Token::StringArray(
                StaticStringToken::new("Hello".to_string(), Some(5)),
            )),
        ];

        assert_eq!(generated_tokens, expected_tokens)
    }

    #[test]
    fn test_token_generation_fail_type_mismatch() {
        for (arg_type, value, error_msg) in [
            (Type::U8, "false", "invalid digit found in string"),
            (Type::U16, "false", "invalid digit found in string"),
            (Type::U32, "false", "invalid digit found in string"),
            (Type::U64, "false", "invalid digit found in string"),
            (Type::U128, "false", "invalid digit found in string"),
            (Type::U256, "false", "Invalid value for U256"),
            (Type::B256, "false", "Odd number of digits"),
            (Type::B256, "0x123", "Odd number of digits"),
            (
                Type::Bool,
                "Hello",
                "provided string was not `true` or `false`",
            ),
        ] {
            assert_eq!(
                Token::from_type_and_value(&arg_type, value)
                    .expect_err("should panic")
                    .to_string(),
                error_msg
            );
        }
    }

    #[test]
    fn test_token_generation_fail_out_of_range() {
        for (arg_type, value, error_msg) in [
            (Type::U8, "256", "number too large to fit in target type"),
            (Type::U16, "65536", "number too large to fit in target type"),
            (
                Type::U32,
                "4294967296",
                "number too large to fit in target type",
            ),
            (
                Type::U64,
                "18446744073709551616",
                "number too large to fit in target type",
            ),
            (
                Type::U128,
                "340282366920938463463374607431768211456",
                "number too large to fit in target type",
            ),
            (
                Type::U256,
                "115792089237316195423570985008687907853269984665640564039457584007913129639936",
                "Invalid value for U256",
            ),
            (
                Type::B256,
                "0x10000000000000000000000000000000000000000000000000000000000000000",
                "Odd number of digits",
            ),
        ] {
            assert_eq!(
                Token::from_type_and_value(&arg_type, value)
                    .expect_err("should panic")
                    .to_string(),
                error_msg
            );
        }
    }

    #[test]
    fn test_type_generation_success() {
        let possible_type_list = [
            "()", "u8", "u16", "u32", "u64", "u128", "u256", "b256", "bool", "String", "str[5]",
        ];
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
            Type::U128,
            Type::U256,
            Type::B256,
            Type::Bool,
            Type::String,
            Type::StringArray(5),
        ];
        assert_eq!(types, expected_types)
    }

    #[test]
    #[should_panic(expected = "u2 type is not supported.")]
    fn test_type_generation_fail_invalid_type() {
        let invalid_type_str = "u2";
        Type::from_str(invalid_type_str).unwrap();
    }

    #[test]
    #[should_panic(expected = "str[abc] type is not supported.")]
    fn test_type_generation_fail_invalid_string_slice() {
        let invalid_type_str = "str[abc]";
        Type::from_str(invalid_type_str).unwrap();
    }

    #[test]
    fn test_try_from_full_type_application() {
        let type_application = FullTypeApplication {
            type_decl: FullTypeDeclaration {
                type_field: "u8".to_string(),
                components: Default::default(),
                type_parameters: Default::default(),
            },
            name: "none".to_string(),
            type_arguments: vec![],
        };
        let generated_type = Type::try_from(&type_application).unwrap();
        assert_eq!(generated_type, Type::U8);
    }

    #[test]
    #[should_panic(expected = "str[abc] type is not supported.")]
    fn test_try_from_full_type_application_invalid_string_slice() {
        let type_application = FullTypeApplication {
            type_decl: FullTypeDeclaration {
                type_field: "str[abc]".to_string(),
                components: Default::default(),
                type_parameters: Default::default(),
            },
            name: "none".to_string(),
            type_arguments: vec![],
        };
        Type::try_from(&type_application).unwrap();
    }
}
