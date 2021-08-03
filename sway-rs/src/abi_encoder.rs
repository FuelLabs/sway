use sha3::{Digest, Keccak256};
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
    pub encoded_sway_function: Vec<u8>,
}

impl ABIEncoder {
    pub fn new(sway_function: String) -> Self {
        Self {
            encoded_sway_function: Self::function_selector(sway_function).to_vec(),
        }
    }

    // TODO: write docs
    pub fn encode(mut self, args: &Vec<Token>) -> Result<Vec<u8>, ABIError> {
        for arg in args {
            match arg {
                Token::U8(arg_u8) => self.encoded_sway_function.extend(pad_u8(arg_u8)),
                Token::U16(arg_u16) => self.encoded_sway_function.extend(pad_u16(arg_u16)),
                Token::U32(arg_u32) => self.encoded_sway_function.extend(pad_u32(arg_u32)),
                Token::U64(arg_u64) => self.encoded_sway_function.extend(arg_u64.to_be_bytes()),
                Token::Byte(arg_byte) => self.encoded_sway_function.extend(pad_u8(arg_byte)),
                Token::Bool(arg_bool) => match arg_bool {
                    true => {
                        let i = 1 as u8;

                        self.encoded_sway_function.extend(pad_u8(&i))
                    }
                    false => {
                        let i = 0 as u8;

                        self.encoded_sway_function.extend(pad_u8(&i))
                    }
                },
                // TODO: We might need to check whether the digest being passed to this function
                // is smaller than 32 bytes or not. If it is, we might wanna do some left padding.
                Token::Bytes32(arg_bytes32) => self.encoded_sway_function.extend(arg_bytes32),
            };
        }
        Ok(self.encoded_sway_function.into())
    }

    pub fn function_selector(signature: String) -> Word {
        let mut hasher = Keccak256::new();
        hasher.update(signature.as_bytes());
        let result = hasher.finalize();

        let mut output = [0u8; 4];

        // Read 4 bytes from it.
        let mut handle = result.take(4);

        handle.read(&mut output).unwrap();

        // Add padding to the previously extract 4 bytes from the function selector
        let mut padding = [0u8; 8];
        padding[4..8].copy_from_slice(&output);

        padding
    }
}

/// Converts a u8 to a right aligned array of 8 bytes.
pub fn pad_u8(value: &u8) -> Word {
    let mut padded = [0u8; 8];
    padded[7..8].copy_from_slice(&value.to_be_bytes());
    padded
}

/// Converts a u16 to a right aligned array of 8 bytes.
pub fn pad_u16(value: &u16) -> Word {
    let mut padded = [0u8; 8];
    padded[6..8].copy_from_slice(&value.to_be_bytes());
    padded
}

/// Converts a u32 to a right aligned array of 8 bytes.
pub fn pad_u32(value: &u32) -> Word {
    let mut padded = [0u8; 8];
    println!("value: {:x?}\n", value.to_le_bytes());
    println!("value.to_be_bytes(): {:x?}\n", value.to_be_bytes());
    padded[4..8].copy_from_slice(&value.to_be_bytes());
    padded
}
