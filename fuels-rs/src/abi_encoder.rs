use crate::tokens::Token;
use crate::types;
use sha2::{Digest, Sha256};
use types::ByteArray;

use crate::errors::Error;

pub struct ABIEncoder {
    pub function_selector: ByteArray,
    pub encoded_args: Vec<u8>,
}

impl ABIEncoder {
    pub fn new() -> Self {
        Self {
            function_selector: [0; 8],
            encoded_args: Vec::new(),
        }
    }

    pub fn new_with_fn_selector(signature: &[u8]) -> Self {
        Self {
            function_selector: Self::encode_function_selector(signature),
            encoded_args: Vec::new(),
        }
    }

    /// Encode takes an array of `Token`s, encodes these tokens, and returns the
    /// raw bytes (as a Vec<u8>) that represent the encoded tokens.
    /// The encoding follows the ABI specs defined
    /// [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md)
    pub fn encode(&mut self, args: &[Token]) -> Result<Vec<u8>, Error> {
        for arg in args {
            match arg {
                Token::U8(arg_u8) => self.encoded_args.extend(types::pad_u8(arg_u8)),
                Token::U16(arg_u16) => self.encoded_args.extend(types::pad_u16(arg_u16)),
                Token::U32(arg_u32) => self.encoded_args.extend(types::pad_u32(arg_u32)),
                Token::U64(arg_u64) => self.encoded_args.extend(arg_u64.to_be_bytes()),
                Token::Byte(arg_byte) => self.encoded_args.extend(types::pad_u8(arg_byte)),
                Token::Bool(arg_bool) => {
                    self.encoded_args
                        .extend(types::pad_u8(if *arg_bool { &1 } else { &0 }))
                }
                Token::B256(arg_bits256) => self.encoded_args.extend(arg_bits256),
                Token::Array(arg_array) => {
                    // Recursively encode the array of Tokens
                    self.encode(arg_array)?;
                }
                Token::String(arg_string) => {
                    self.encoded_args.extend(types::pad_string(arg_string))
                }
                Token::Struct(arg_struct) => {
                    for property in arg_struct.into_iter() {
                        self.encode(&[property.to_owned()])?;
                    }
                }
                Token::Enum(arg_enum) => {
                    // Encode the discriminant of the enum
                    self.encoded_args.extend(types::pad_u8(&arg_enum.0));
                    // Encode the Token within the enum
                    self.encode(&[arg_enum.1.to_owned()])?;
                }
            };
        }
        Ok(self.encoded_args.clone().into())
    }

    pub fn encode_function_selector(signature: &[u8]) -> ByteArray {
        let mut hasher = Sha256::new();
        hasher.update(signature);
        let result = hasher.finalize();

        let mut output = types::ByteArray::default();

        (&mut output[4..]).copy_from_slice(&result[..4]);

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_function_signature() {
        let sway_fn = "entry_one(u64)";

        let result = ABIEncoder::encode_function_selector(sway_fn.as_bytes());

        println!(
            "Encoded function selector for ({}): {:#0x?}",
            sway_fn, result
        );

        assert_eq!(result, [0x0, 0x0, 0x0, 0x0, 0x0c, 0x36, 0xcb, 0x9c]);
    }

    #[test]
    fn encode_function_with_u32_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"u32"}],
        //         "name":"entry_one",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "entry_one(u32)";
        let arg = Token::U32(u32::MAX);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xb7, 0x9e, 0xf7, 0x43];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_u32_type_multiple_args() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"first","type":"u32"},{"name":"second","type":"u32"}],
        //         "name":"takes_two",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_two(u32,u32)";
        let first = Token::U32(u32::MAX);
        let second = Token::U32(u32::MAX);

        let args: Vec<Token> = vec![first, second];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xa7, 0x07, 0xb0, 0x8e];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_u64_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"u64"}],
        //         "name":"entry_one",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "entry_one(u64)";
        let arg = Token::U64(u64::MAX);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x0c, 0x36, 0xcb, 0x9c];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_bool_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"bool"}],
        //         "name":"bool_check",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "bool_check(bool)";
        let arg = Token::Bool(true);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x66, 0x8f, 0xff, 0x58];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_two_different_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"first","type":"u32"},{"name":"second","type":"bool"}],
        //         "name":"takes_two_types",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_two_types(u32,bool)";
        let first = Token::U32(u32::MAX);
        let second = Token::Bool(true);

        let args: Vec<Token> = vec![first, second];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xf5, 0x40, 0x73, 0x2b];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}) {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_byte_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"byte"}],
        //         "name":"takes_one_byte",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_one_byte(byte)";
        let arg = Token::Byte(u8::MAX);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x2e, 0xe3, 0xce, 0x1f];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_bits256_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"b256"}],
        //         "name":"takes_bits256",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_bits256(b256)";

        let mut hasher = Sha256::new();
        hasher.update("test string".as_bytes());

        let arg = hasher.finalize();

        let arg = Token::B256(arg.into());

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [
            0xd5, 0x57, 0x9c, 0x46, 0xdf, 0xcc, 0x7f, 0x18, 0x20, 0x70, 0x13, 0xe6, 0x5b, 0x44,
            0xe4, 0xcb, 0x4e, 0x2c, 0x22, 0x98, 0xf4, 0xac, 0x45, 0x7b, 0xa8, 0xf8, 0x27, 0x43,
            0xf3, 0x1e, 0x93, 0xb,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x01, 0x49, 0x42, 0x96];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_array_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"u8[3]"}],
        //         "name":"takes_integer_array",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_integer_array(u8[3])";

        // Keeping the construction of the arguments array separate for better readability.
        let first = Token::U8(1);
        let second = Token::U8(2);
        let third = Token::U8(3);

        let arg = vec![first, second, third];
        let arg_array = Token::Array(arg);

        let args: Vec<Token> = vec![arg_array];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x2c, 0x5a, 0x10, 0x2e];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_string_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"str[12]"}],
        //         "name":"takes_string",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_string(str[23])";

        let args: Vec<Token> = vec![Token::String("This is a full sentence".into())];

        let expected_encoded_abi = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x66, 0x75, 0x6c, 0x6c,
            0x20, 0x73, 0x65, 0x6e, 0x74, 0x65, 0x6e, 0x63, 0x65, 0x00,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xd5, 0x6e, 0x76, 0x51];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_struct() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"MyStruct"}],
        //         "name":"takes_my_struct",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_my_struct(MyStruct)";

        // Sway struct:
        // struct MyStruct {
        //     foo: u8,
        //     bar: bool,
        // }

        let foo = Token::U8(1);
        let bar = Token::Bool(true);

        // Create the custom struct token using the array of tuples above
        let arg = Token::Struct(vec![foo, bar]);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xa8, 0x1e, 0x8d, 0xd7];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_enum() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"MyEnum"}],
        //         "name":"takes_my_enum",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_my_enum(MyEnum)";

        // Sway enum:
        // enum MyEnum {
        //     x: u32,
        //     y: bool,
        // }

        // Create a tuple with the Enum discriminant (`0` in this case)
        // And the value matching the discriminant type.
        let val = Box::new((0, Token::U32(42)));

        // Create the custom enum token using the array of the tuple above
        let arg = Token::Enum(val);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2a,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x35, 0x5c, 0xa6, 0xfa];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_nested_structs() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"Foo"}],
        //         "name":"takes_my_nested_struct",
        //         "outputs": []
        //     }
        // ]
        // "#;

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

        let sway_fn = "takes_my_nested_struct(Foo)";

        let args: Vec<Token> = vec![Token::Struct(vec![
            Token::U16(10),
            Token::Struct(vec![
                Token::Bool(true),
                Token::Array(vec![Token::U8(1), Token::U8(2)]),
            ]),
        ])];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xea, 0x0a, 0xfd, 0x23];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_comprehensive_function() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type": "contract",
        //         "inputs": [
        //         {
        //             "name": "arg",
        //             "type": "Foo"
        //         },
        //         {
        //             "name": "arg2",
        //             "type": "u8[2]"
        //         },
        //         {
        //             "name": "arg3",
        //             "type": "b256"
        //         },
        //         {
        //             "name": "arg",
        //             "type": "str[23]"
        //         }
        //         ],
        //         "name": "long_function",
        //         "outputs": []
        //     }
        // ]
        // "#;

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

        let sway_fn = "long_function(Foo,u8[2],b256,str[23])";

        let foo = Token::Struct(vec![
            Token::U16(10),
            Token::Struct(vec![
                Token::Bool(true),
                Token::Array(vec![Token::U8(1), Token::U8(2)]),
            ]),
        ]);

        let u8_arr = Token::Array(vec![Token::U8(1), Token::U8(2)]);

        let mut hasher = Sha256::new();
        hasher.update("test string".as_bytes());

        let b256 = Token::B256(hasher.finalize().into());

        let s = Token::String("This is a full sentence".into());

        let args: Vec<Token> = vec![foo, u8_arr, b256, s];

        let expected_encoded_abi = [
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

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x10, 0x93, 0xb2, 0x12];

        let mut abi_encoder = ABIEncoder::new_with_fn_selector(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!("Encoded ABI for ({}): {:#0x?}", sway_fn, encoded);

        println!(
            "abi_encoder.function_selector: {:#0x?}\n",
            abi_encoder.function_selector
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }
}
