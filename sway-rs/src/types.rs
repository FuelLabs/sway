#![allow(dead_code)] // Temporary while it's a WIP.
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, ToString};

pub type Word = [u8; 8];
pub const WORD_SIZE: isize = 8;

pub type Bits256 = [u8; 32];
pub type EnumSelector = (u8, Token);

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

impl<'a> Default for Token {
    fn default() -> Self {
        Token::U8(0)
    }
}

// Experimental
#[derive(Debug, Clone, EnumString, ToString)]
#[strum(ascii_case_insensitive)]
pub enum ParamType {
    U8,
    U16,
    U32,
    U64,
    Bool,
    Byte,
    B256,
    Array(Box<ParamType>, usize),
    #[strum(serialize = "str")]
    String(usize),
    Struct(Vec<ParamType>),
    Enum(Vec<ParamType>),
}

impl Default for ParamType {
    fn default() -> Self {
        ParamType::U8
    }
}

pub type JsonABI = Vec<Entry>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    #[serde(rename = "type")]
    pub type_field: String,
    pub inputs: Vec<Property>,
    pub name: String,
    pub outputs: Vec<Property>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub components: Option<Vec<Property>>, // Used for custom types
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Component {
//     pub name: String,
//     #[serde(rename = "type")]
//     pub type_name: String,
// }

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

pub fn pad_string(s: &str) -> Vec<u8> {
    // Computing the number of zero bytes we need to pad
    // the string in order to be left-aligned to 8 bytes.
    // Formula is: (N - L) % N
    // Where N is word size and L is the length of the str.
    let bytes_to_pad = (WORD_SIZE - s.len() as isize).rem_euclid(WORD_SIZE);

    let mut res = s.as_bytes().to_owned();

    res.extend_from_slice(&vec![0; bytes_to_pad as usize]);

    res
}
