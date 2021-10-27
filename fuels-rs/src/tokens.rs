use crate::types::{Bits256, EnumSelector};
use std::fmt;
use strum_macros::EnumString;

#[derive(Clone, Debug)]
pub struct InvalidOutputType(pub String);

// Sway types
#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Token {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Bool(bool),
    Byte(u8),
    B256(Bits256),
    Array(Vec<Token>),
    String(String),
    Struct(Vec<Token>),
    Enum(Box<EnumSelector>),
}

/// Simplified output type for single value.
pub trait Tokenizable {
    /// Converts a `Token` into expected type.
    fn from_token(token: Token) -> Result<Self, InvalidOutputType>
    where
        Self: Sized;
    /// Converts a specified type back into token.
    fn into_token(self) -> Token;
}

impl Tokenizable for Token {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        Ok(token)
    }
    fn into_token(self) -> Token {
        self
    }
}

impl Tokenizable for bool {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Bool(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `bool`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::Bool(self)
    }
}

impl Tokenizable for String {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::String(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `String`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::String(self)
    }
}

impl Tokenizable for Bits256 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::B256(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `String`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::B256(self)
    }
}

impl<T: Tokenizable> Tokenizable for Vec<T> {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Array(data) => {
                let mut v: Vec<T> = Vec::new();
                for tok in data {
                    v.push(T::from_token(tok.clone()).unwrap());
                }
                return Ok(v);
            }
            other => Err(InvalidOutputType(format!("Expected `T`, got {:?}", other))),
        }
    }
    fn into_token(self) -> Token {
        let mut v: Vec<Token> = Vec::new();
        for t in self {
            let tok = T::into_token(t);
            v.push(tok);
        }
        Token::Array(v)
    }
}

impl Tokenizable for u8 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::U8(data) => Ok(data),
            other => Err(InvalidOutputType(format!("Expected `u8`, got {:?}", other))),
        }
    }
    fn into_token(self) -> Token {
        Token::U8(self)
    }
}

impl Tokenizable for u16 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::U16(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `u16`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::U16(self)
    }
}

impl Tokenizable for u32 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::U32(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `u32`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::U32(self)
    }
}

impl Tokenizable for u64 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::U64(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `u64`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::U64(self)
    }
}

/// Output type possible to deserialize from Contract ABI
pub trait Detokenize {
    /// Creates a new instance from parsed ABI tokens.
    fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType>
    where
        Self: Sized;
}

impl Detokenize for () {
    fn from_tokens(_: Vec<Token>) -> std::result::Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl<T: Tokenizable> Detokenize for T {
    fn from_tokens(mut tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
        let token = match tokens.len() {
            0 => Token::Struct(vec![]),
            1 => tokens.remove(0),
            _ => Token::Struct(tokens),
        };

        Self::from_token(token)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> Default for Token {
    fn default() -> Self {
        Token::U8(0)
    }
}
