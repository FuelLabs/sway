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
            other => Err(InvalidOutputType(format!("Expected `u8`, got {:?}", other))),
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
            other => Err(InvalidOutputType(format!("Expected `u8`, got {:?}", other))),
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
            other => Err(InvalidOutputType(format!("Expected `u8`, got {:?}", other))),
        }
    }
    fn into_token(self) -> Token {
        Token::U64(self)
    }
}

/// Tokens conversion trait
pub trait Tokenize {
    /// Convert to list of tokens
    fn into_tokens(self) -> Vec<Token>;
}

impl<'a> Tokenize for &'a [Token] {
    fn into_tokens(self) -> Vec<Token> {
        flatten_tokens(self.to_vec())
    }
}

impl<T: Tokenizable> Tokenize for T {
    fn into_tokens(self) -> Vec<Token> {
        flatten_tokens(vec![self.into_token()])
    }
}

impl Tokenize for () {
    fn into_tokens(self) -> Vec<Token> {
        vec![]
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

/// Helper for flattening non-nested tokens into their inner
/// types, e.g. (A, B, C ) would get tokenized to Tuple([A, B, C])
/// when in fact we need [A, B, C].
fn flatten_tokens(mut tokens: Vec<Token>) -> Vec<Token> {
    if tokens.len() == 1 {
        // flatten the tokens if required
        // and there is no nesting
        match tokens.remove(0) {
            Token::Struct(inner) => inner,
            other => vec![other],
        }
    } else {
        tokens
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
