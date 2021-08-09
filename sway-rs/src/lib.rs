#[cfg(test)]
mod abi_encoder;

mod tests {
    use super::*;

    use sha2::{Digest, Sha256};

    #[test]
    fn encode_function_signature() {
        let sway_fn = "entry_one(u64)";

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let result = abi_encoder.function_selector;

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
        //         "type":"contract",
        //         "inputs": [{"name":"arg","type":"u32"}],
        //         "name":"entry_one",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "entry_one(u32)";
        let arg = abi_encoder::Token::U32(u32::MAX);

        let args: Vec<abi_encoder::Token> = vec![arg];

        // Expected encoded ABI:
        // "0x0000ffffffff";

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xb7, 0x9e, 0xf7, 0x43];

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, arg, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_u32_type_multiple_args() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"contract",
        //         "inputs": [{"name":"first","type":"u32"},{"name":"second","type":"u32"}],
        //         "name":"takes_two",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_two(u32,u32)";
        let first = abi_encoder::Token::U32(u32::MAX);
        let second = abi_encoder::Token::U32(u32::MAX);

        let args: Vec<abi_encoder::Token> = vec![first, second];

        // Expected encoded ABI:
        // "0x0000ffffffff0000ffffffff";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xa7, 0x07, 0xb0, 0x8e];

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}, {:?}): {:#0x?}",
            sway_fn, first, second, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_u64_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"contract",
        //         "inputs": [{"name":"arg","type":"u64"}],
        //         "name":"entry_one",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "entry_one(u64)";
        let arg = abi_encoder::Token::U64(u64::MAX);

        let args: Vec<abi_encoder::Token> = vec![arg];

        // Expected encoded ABI:
        // "0xffffffffffffffff";

        let expected_encoded_abi = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x0c, 0x36, 0xcb, 0x9c];

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, arg, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_bool_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"contract",
        //         "inputs": [{"name":"arg","type":"bool"}],
        //         "name":"bool_check",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "bool_check(bool)";
        let arg = abi_encoder::Token::Bool(true);

        let args: Vec<abi_encoder::Token> = vec![arg];

        // Expected encoded ABI:
        // "0x00000001";

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x66, 0x8f, 0xff, 0x58];

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, arg, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_two_different_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"contract",
        //         "inputs": [{"name":"first","type":"u32"},{"name":"second","type":"bool"}],
        //         "name":"takes_two_types",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_two_types(u32,bool)";
        let first = abi_encoder::Token::U32(u32::MAX);
        let second = abi_encoder::Token::Bool(true);

        let args: Vec<abi_encoder::Token> = vec![first, second];

        // Expected encoded ABI:
        // "0x0000ffffffff00000001";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xf5, 0x40, 0x73, 0x2b];

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}, {:?}): {:#0x?}",
            sway_fn, first, second, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_byte_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"contract",
        //         "inputs": [{"name":"arg","type":"byte"}],
        //         "name":"takes_one_byte",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_one_byte(byte)";
        let arg = abi_encoder::Token::Byte(u8::MAX);

        let args: Vec<abi_encoder::Token> = vec![arg];

        // Expected encoded ABI:
        // "0x0000000ff";

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x2e, 0xe3, 0xce, 0x1f];

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, arg, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }

    #[test]
    fn encode_function_with_bytes32_type() {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"contract",
        //         "inputs": [{"name":"arg","type":"bytes32"}],
        //         "name":"takes_bytes32",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let sway_fn = "takes_bytes32(bytes32)";

        let mut hasher = Sha256::new();
        hasher.update("test string".as_bytes());

        let arg = hasher.finalize();

        let arg = abi_encoder::Token::Bytes32(arg.into());

        let args: Vec<abi_encoder::Token> = vec![arg];

        // Expected encoded ABI:
        // "0xd5579c46dfcc7f18207013e65b44e4cb4e2c2298f4ac457ba8f82743f31e930b";

        let expected_encoded_abi = [
            0xd5, 0x57, 0x9c, 0x46, 0xdf, 0xcc, 0x7f, 0x18, 0x20, 0x70, 0x13, 0xe6, 0x5b, 0x44,
            0xe4, 0xcb, 0x4e, 0x2c, 0x22, 0x98, 0xf4, 0xac, 0x45, 0x7b, 0xa8, 0xf8, 0x27, 0x43,
            0xf3, 0x1e, 0x93, 0xb,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x8f, 0x72, 0x18, 0x52];

        let mut abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.as_bytes());

        let encoded = abi_encoder.encode(args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, arg, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(abi_encoder.function_selector, expected_function_selector);
    }
}
