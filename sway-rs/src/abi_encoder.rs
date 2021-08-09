use sha2::{Digest, Sha256};
use std::io::Read;
use thiserror::Error;

pub type Word = [u8; 8];

pub type Bytes32 = [u8; 32];
// Sway types
#[derive(Debug, Copy, Clone)]
pub enum Token {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Bool(bool),
    Byte(u8),
    Bytes32(Bytes32),
}

// Currently not being used
pub enum PadDirection {
    Right,
    Left,
}

#[derive(Debug, Clone, Error)]
pub enum ABIError {
    #[error("Failed to decode")]
    DecodingError,

    #[error("missing or wrong function selector")]
    WrongSelector,
}

pub struct ABIEncoder {
    pub function_selector: Word,
    pub encoded_args: Vec<u8>,
}

impl ABIEncoder {
    pub fn new(sway_function: &[u8]) -> Self {
        Self {
            function_selector: Self::function_selector(sway_function),
            encoded_args: Vec::new(),
        }
    }

    // TODO: write docs
    pub fn encode(&mut self, args: Vec<Token>) -> Result<Vec<u8>, ABIError> {
        for arg in args {
            match arg {
                Token::U8(arg_u8) => self.encoded_args.extend(pad_u8(arg_u8)),
                Token::U16(arg_u16) => self.encoded_args.extend(pad_u16(arg_u16)),
                Token::U32(arg_u32) => self.encoded_args.extend(pad_u32(arg_u32)),
                Token::U64(arg_u64) => self.encoded_args.extend(arg_u64.to_be_bytes()),
                Token::Byte(arg_byte) => self.encoded_args.extend(pad_u8(arg_byte)),
                Token::Bool(arg_bool) => {
                    self.encoded_args
                        .extend(pad_u8(if arg_bool { 1 } else { 0 }))
                }
                // TODO: We might need to check whether the digest being passed to this function
                // is smaller than 32 bytes or not. If it is, we might wanna do some left padding.
                Token::Bytes32(arg_bytes32) => self.encoded_args.extend(arg_bytes32),
            };
        }
        Ok(self.encoded_args.clone().into())
    }

    pub fn function_selector(signature: &[u8]) -> Word {
        let mut hasher = Sha256::new();
        hasher.update(signature);
        let result = hasher.finalize();

        let mut output = Word::default();

        (&mut output[4..]).copy_from_slice(&result[..4]);

        output
    }
}

/// Converts a u8 to a right aligned array of 8 bytes.
pub fn pad_u8(value: u8) -> Word {
    let mut padded = Word::default();
    padded[7] = value;
    padded
}

/// Converts a u16 to a right aligned array of 8 bytes.
pub fn pad_u16(value: u16) -> Word {
    let mut padded = Word::default();
    padded[6..].copy_from_slice(&value.to_be_bytes());
    padded
}

/// Converts a u32 to a right aligned array of 8 bytes.
pub fn pad_u32(value: u32) -> Word {
    let mut padded = [0u8; 8];
    padded[4..].copy_from_slice(&value.to_be_bytes());
    padded
}
