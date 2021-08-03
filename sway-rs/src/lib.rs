#[cfg(test)]
mod abi_encoder;

mod tests {
    use super::*;

    use sha3::{Digest, Keccak256};

    #[test]
    fn encode_function_signature() {
        let sway_fn = "entry_one(u64)";

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let result = abi_encoder.encoded_sway_function;

        println!(
            "Encoded function selector for ({}): {:#0x?}",
            sway_fn, result
        );

        assert_eq!(result, [0x0, 0x0, 0x0, 0x0, 0x67, 0x19, 0xaf, 0xac]);
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
        // "00003d62125d0000ffffffff";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x3d, 0x62, 0x12, 0x5d, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff,
        ];

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, args, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
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
        // "00001b6f3f790000ffffffff0000ffffffff";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x1b, 0x6f, 0x3f, 0x79, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff,
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff,
        ];

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, args, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
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
        // "00006719afacffffffffffffffff";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x67, 0x19, 0xaf, 0xac, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff,
        ];

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, args, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
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
        // "00004f0bebd300000001";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x4f, 0xb, 0xeb, 0xd3, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1,
        ];

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, args, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
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
        // "0000494599540000ffffffff00000001";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x49, 0x45, 0x99, 0x54, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1,
        ];

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, args, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
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
        // "0000996725860000ffffffff";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x99, 0x67, 0x25, 0x86, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff,
        ];

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, args, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
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

        let mut hasher = Keccak256::new();
        hasher.update("test string".as_bytes());

        let arg = hasher.finalize();

        let arg = abi_encoder::Token::Bytes32(arg.into());

        let args: Vec<abi_encoder::Token> = vec![arg];

        // Expected encoded ABI:
        // "0000ff1e564dc7fd1d987ada439fc085cfa3c49416cf2b504ac50151e3c2335d60595cb90745";

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0x1e, 0x56, 0x4d, 0xc7, 0xfd, 0x1d, 0x98, 0x7a, 0xda, 0x43,
            0x9f, 0xc0, 0x85, 0xcf, 0xa3, 0xc4, 0x94, 0x16, 0xcf, 0x2b, 0x50, 0x4a, 0xc5, 0x1,
            0x51, 0xe3, 0xc2, 0x33, 0x5d, 0x60, 0x59, 0x5c, 0xb9, 0x7, 0x45,
        ];

        let abi_encoder = abi_encoder::ABIEncoder::new(sway_fn.into());

        let encoded = abi_encoder.encode(&args).unwrap();

        println!(
            "Encoded ABI for ({}) with args ({:?}): {:#0x?}",
            sway_fn, args, encoded
        );

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
    }
}
