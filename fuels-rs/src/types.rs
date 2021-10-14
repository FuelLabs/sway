use crate::errors::Error;
use crate::tokens::Token;
use anyhow::Result;
use fuel_types::bytes::padded_len;
use fuel_types::Word;
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, ToString};

pub type ByteArray = [u8; 8];
pub type Selector = ByteArray;
pub type Bits256 = [u8; 32];
pub type EnumSelector = (u8, Token);
pub const WORD_SIZE: usize = core::mem::size_of::<Word>();

#[derive(Debug, Clone, EnumString, ToString, PartialEq, Eq)]
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
    // Disabling EnumString on these 2 types because
    // they are more complex to parse
    #[strum(disabled)]
    Struct(Vec<ParamType>),
    #[strum(disabled)]
    Enum(Vec<ParamType>),
}

impl Default for ParamType {
    fn default() -> Self {
        ParamType::U8
    }
}

pub type JsonABI = Vec<Function>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
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

/// Expands a [`ParamType`] into a TokenStream.
/// Used to expand functions when generating type-safe bindings of a JSON ABI.
pub fn expand_type(kind: &ParamType) -> Result<TokenStream, Error> {
    match kind {
        ParamType::U8 | ParamType::Byte => Ok(quote! { u8 }),
        ParamType::U16 => Ok(quote! { u16 }),
        ParamType::U32 => Ok(quote! { u32 }),
        ParamType::U64 => Ok(quote! { u64 }),
        ParamType::Bool => Ok(quote! { bool }),
        ParamType::B256 => Ok(quote! { [u8; 32] }),
        ParamType::String(_) => Ok(quote! { String }),
        ParamType::Array(t, size) => {
            let inner = expand_type(t)?;
            Ok(quote! { [#inner; #size] })
        }
        ParamType::Struct(members) => {
            if members.is_empty() {
                return Err(Error::InvalidData);
            }
            let members = members
                .iter()
                .map(|member| expand_type(member))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { (#(#members,)*) })
        }
        ParamType::Enum(members) => {
            if members.is_empty() {
                return Err(Error::InvalidData);
            }
            let members = members
                .iter()
                .map(|member| expand_type(member))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { (#(#members,)*) })
        }
    }
}

/// Converts a u8 to a right aligned array of 8 bytes.
pub fn pad_u8(value: &u8) -> ByteArray {
    let mut padded = ByteArray::default();
    padded[7] = *value;
    padded
}

/// Converts a u16 to a right aligned array of 8 bytes.
pub fn pad_u16(value: &u16) -> ByteArray {
    let mut padded = ByteArray::default();
    padded[6..].copy_from_slice(&value.to_be_bytes());
    padded
}

/// Converts a u32 to a right aligned array of 8 bytes.
pub fn pad_u32(value: &u32) -> ByteArray {
    let mut padded = [0u8; 8];
    padded[4..].copy_from_slice(&value.to_be_bytes());
    padded
}

pub fn pad_string(s: &str) -> Vec<u8> {
    let pad = padded_len(s.as_bytes()) - s.len();

    let mut padded = s.as_bytes().to_owned();

    padded.extend_from_slice(&vec![0; pad]);

    padded
}
