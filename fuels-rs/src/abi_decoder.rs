use crate::errors::Error;
use crate::tokens::Token;
use crate::types::{Bits256, ByteArray, ParamType, WORD_SIZE};
use fuel_types::bytes::padded_len;
use std::convert::TryInto;
use std::str;

#[derive(Debug, Clone)]
struct DecodeResult {
    token: Token,
    new_offset: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct ABIDecoder {}

impl ABIDecoder {
    pub fn new() -> Self {
        ABIDecoder {}
    }

    /// Decode takes an array of `ParamType` and the encoded data as raw bytes
    /// and returns a vector of `Token`s containing the decoded values.
    /// Note that the order of the types in the `types` array needs to match the order
    /// of the expected values/types in `data`.
    /// You can find comprehensive examples in the tests for this module.
    pub fn decode<'a>(&mut self, types: &[ParamType], data: &'a [u8]) -> Result<Vec<Token>, Error> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut offset = 0;
        for param in types {
            let res = self.decode_param(param, data, offset)?;
            offset = res.new_offset;
            tokens.push(res.token);
        }

        Ok(tokens)
    }

    fn decode_param<'a>(
        self,
        param: &ParamType,
        data: &'a [u8],
        offset: usize,
    ) -> Result<DecodeResult, Error> {
        match &*param {
            ParamType::U8 => {
                let slice = peek_word(data, offset)?;

                let result = DecodeResult {
                    token: Token::U8(u8::from_be_bytes(slice[7..8].try_into().unwrap())),
                    new_offset: offset + 8,
                };

                Ok(result)
            }
            ParamType::U16 => {
                let slice = peek_word(data, offset)?;

                let result = DecodeResult {
                    token: Token::U16(u16::from_be_bytes(slice[6..8].try_into().unwrap())),
                    new_offset: offset + 8,
                };

                Ok(result)
            }
            ParamType::U32 => {
                let slice = peek_word(data, offset)?;

                let result = DecodeResult {
                    token: Token::U32(u32::from_be_bytes(slice[4..8].try_into().unwrap())),
                    new_offset: offset + 8,
                };

                Ok(result)
            }
            ParamType::U64 => {
                let slice = peek_word(data, offset)?;

                let result = DecodeResult {
                    token: Token::U64(u64::from_be_bytes(slice.try_into().unwrap())),
                    new_offset: offset + 8,
                };

                Ok(result)
            }
            ParamType::Bool => {
                // Grab last byte of the word and compare it to 0x00
                let b = peek_word(data, offset)?.last().unwrap() != &0u8;

                let result = DecodeResult {
                    token: Token::Bool(b),
                    new_offset: offset + 8,
                };

                Ok(result)
            }
            ParamType::Byte => {
                // Grab last byte of the word and compare it to 0x00
                let byte = peek_word(data, offset)?.last().unwrap().clone();

                let result = DecodeResult {
                    token: Token::Byte(byte),
                    new_offset: offset + 8,
                };

                Ok(result)
            }
            ParamType::B256 => {
                let b256: Bits256 = peek(data, offset, 32)?.try_into().unwrap();

                let result = DecodeResult {
                    token: Token::B256(b256),
                    new_offset: offset + 32,
                };

                Ok(result)
            }
            ParamType::String(length) => {
                let encoded_str = peek(data, offset, *length)?.try_into().unwrap();

                let decoded = str::from_utf8(encoded_str)?;

                let result = DecodeResult {
                    token: Token::String(decoded.to_string()),
                    new_offset: offset + padded_len(encoded_str),
                };

                Ok(result)
            }
            ParamType::Array(ref t, length) => {
                let mut tokens = vec![];
                let mut new_offset = offset;

                for _ in 0..*length {
                    let res = self.decode_param(t, data, new_offset)?;
                    new_offset = res.new_offset;
                    tokens.push(res.token);
                }

                let result = DecodeResult {
                    token: Token::Array(tokens),
                    new_offset,
                };

                Ok(result)
            }
            ParamType::Struct(props) => {
                let mut tokens = vec![];

                let mut new_offset = offset;
                for prop in props {
                    let res = self.decode_param(prop, data, new_offset)?;
                    new_offset = res.new_offset;
                    tokens.push(res.token);
                }

                let result = DecodeResult {
                    token: Token::Struct(tokens),
                    new_offset,
                };

                Ok(result)
            }
            ParamType::Enum(variations) => {
                let discriminant = peek_word(data, offset).unwrap();

                let discriminant = u32::from_be_bytes(discriminant[4..8].try_into().unwrap());

                // Offset + 8 because of the discriminant that we just peeked
                let res = self.decode_param(
                    variations.get(discriminant as usize).unwrap(),
                    data,
                    offset + 8,
                )?;

                let result = DecodeResult {
                    token: Token::Enum(Box::new((discriminant as u8, res.token))),
                    new_offset: res.new_offset,
                };

                Ok(result)
            }
        }
    }
}

fn peek(data: &[u8], offset: usize, len: usize) -> Result<&[u8], Error> {
    if offset + len > data.len() {
        Err(Error::InvalidData)
    } else {
        Ok(&data[offset..(offset + len)])
    }
}

fn peek_word(data: &[u8], offset: usize) -> Result<ByteArray, Error> {
    peek(data, offset, WORD_SIZE as usize).map(|x| {
        let mut out: ByteArray = [0u8; 8];
        out.copy_from_slice(&x[0..8]);
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_int() {
        let types = vec![ParamType::U32];
        let data = [0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff];

        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![Token::U32(u32::MAX)];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_multiple_int() {
        let types = vec![
            ParamType::U32,
            ParamType::U8,
            ParamType::U16,
            ParamType::U64,
        ];
        let data = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff,
        ];

        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![
            Token::U32(u32::MAX),
            Token::U8(u8::MAX),
            Token::U16(u16::MAX),
            Token::U64(u64::MAX),
        ];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_bool() {
        let types = vec![ParamType::Bool, ParamType::Bool];
        let data = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x00,
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![Token::Bool(true), Token::Bool(false)];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_b256() {
        let types = vec![ParamType::B256];
        let data = [
            0xd5, 0x57, 0x9c, 0x46, 0xdf, 0xcc, 0x7f, 0x18, 0x20, 0x70, 0x13, 0xe6, 0x5b, 0x44,
            0xe4, 0xcb, 0x4e, 0x2c, 0x22, 0x98, 0xf4, 0xac, 0x45, 0x7b, 0xa8, 0xf8, 0x27, 0x43,
            0xf3, 0x1e, 0x93, 0xb,
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![Token::B256(data)];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_string() {
        let types = vec![ParamType::String(23), ParamType::String(5)];
        let data = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x66, 0x75, 0x6c, 0x6c,
            0x20, 0x73, 0x65, 0x6e, 0x74, 0x65, 0x6e, 0x63, 0x65, 0x00, 0x48, 0x65, 0x6c, 0x6c,
            0x6f, 0x0, 0x0, 0x0,
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![
            Token::String("This is a full sentence".into()),
            Token::String("Hello".into()),
        ];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }
    #[test]
    fn decode_array() {
        // Create a parameter type for u8[2].
        let types = vec![ParamType::Array(Box::new(ParamType::U8), 2)];
        let data = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2a,
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![Token::Array(vec![Token::U8(255), Token::U8(42)])];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_struct() {
        // Sway struct:
        // struct MyStruct {
        //     foo: u8,
        //     bar: bool,
        // }
        let types = vec![ParamType::Struct(vec![ParamType::U8, ParamType::Bool])];

        let data = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1,
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![Token::Struct(vec![Token::U8(1), Token::Bool(true)])];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_enum() {
        // Sway enum:
        // enum MyEnum {
        //     x: u32,
        //     y: bool,
        // }

        let types = vec![ParamType::Enum(vec![ParamType::U32, ParamType::Bool])];

        // "0" discriminant and 42 enum value
        let data = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2a,
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let expected = vec![Token::Enum(Box::new((0, Token::U32(42))))];
        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_nested_struct() {
        // Sway nested struct:
        // struct Foo {
        //     x: u16,
        //     y: Bar,
        // }
        //
        // struct Bar {
        //     a: bool,
        //     b: u8[2],
        // }

        let nested_struct = ParamType::Struct(vec![
            ParamType::U16,
            ParamType::Struct(vec![
                ParamType::Bool,
                ParamType::Array(Box::new(ParamType::U8), 2),
            ]),
        ]);
        let types = vec![nested_struct];

        let data = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        let my_nested_struct = vec![
            Token::U16(10),
            Token::Struct(vec![
                Token::Bool(true),
                Token::Array(vec![Token::U8(1), Token::U8(2)]),
            ]),
        ];

        let expected = vec![Token::Struct(my_nested_struct)];

        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }

    #[test]
    fn decode_comprehensive() {
        // Sway nested struct:
        // struct Foo {
        //     x: u16,
        //     y: Bar,
        // }
        //
        // struct Bar {
        //     a: bool,
        //     b: u8[2],
        // }

        // Sway fn: long_function(Foo,u8[2],b256,str[23])

        // Parameters
        let nested_struct = ParamType::Struct(vec![
            ParamType::U16,
            ParamType::Struct(vec![
                ParamType::Bool,
                ParamType::Array(Box::new(ParamType::U8), 2),
            ]),
        ]);

        let u8_arr = ParamType::Array(Box::new(ParamType::U8), 2);
        let b256 = ParamType::B256;
        let s = ParamType::String(23);

        let types = vec![nested_struct, u8_arr, b256, s];

        let data = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa, // foo.x == 10u16
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, // foo.y.a == true
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, // foo.b.0 == 1u8
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, // foo.b.1 == 2u8
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, // u8[2].0 == 1u8
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, // u8[2].0 == 2u8
            0xd5, 0x57, 0x9c, 0x46, 0xdf, 0xcc, 0x7f, 0x18, // b256
            0x20, 0x70, 0x13, 0xe6, 0x5b, 0x44, 0xe4, 0xcb, // b256
            0x4e, 0x2c, 0x22, 0x98, 0xf4, 0xac, 0x45, 0x7b, // b256
            0xa8, 0xf8, 0x27, 0x43, 0xf3, 0x1e, 0x93, 0xb, // b256
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, // str[23]
            0x61, 0x20, 0x66, 0x75, 0x6c, 0x6c, 0x20, 0x73, // str[23]
            0x65, 0x6e, 0x74, 0x65, 0x6e, 0x63, 0x65, 0x0, // str[23]
        ];
        let mut decoder = ABIDecoder::new();

        let decoded = decoder.decode(&types, &data).unwrap();

        // Expected tokens
        let foo = Token::Struct(vec![
            Token::U16(10),
            Token::Struct(vec![
                Token::Bool(true),
                Token::Array(vec![Token::U8(1), Token::U8(2)]),
            ]),
        ]);

        let u8_arr = Token::Array(vec![Token::U8(1), Token::U8(2)]);

        let b256 = Token::B256([
            0xd5, 0x57, 0x9c, 0x46, 0xdf, 0xcc, 0x7f, 0x18, 0x20, 0x70, 0x13, 0xe6, 0x5b, 0x44,
            0xe4, 0xcb, 0x4e, 0x2c, 0x22, 0x98, 0xf4, 0xac, 0x45, 0x7b, 0xa8, 0xf8, 0x27, 0x43,
            0xf3, 0x1e, 0x93, 0xb,
        ]);

        let s = Token::String("This is a full sentence".into());

        let expected: Vec<Token> = vec![foo, u8_arr, b256, s];

        assert_eq!(decoded, expected);

        println!(
            "Decoded ABI for ({:#0x?}) with types ({:?}): {:?}",
            data, types, decoded
        );
    }
}
