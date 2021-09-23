use core::panic;
use hex::decode;
use hex::FromHex;
use regex::{Captures, Regex};
use std::convert::TryInto;
use std::str;
use std::str::FromStr;

use crate::{
    abi_decoder::ABIDecoder,
    abi_encoder::ABIEncoder,
    types::{Bits256, JsonABI, ParamType, Property, Token},
};
use serde_json;

// TODO: clean-up this disaster of code
// TODO: improve error handling, error messages
// TODO: Update ABI documentation wrt to JSON for strucs/enums

pub struct ABI {}

impl ABI {
    pub fn new() -> Self {
        ABI {}
    }

    /// Higher-level layer of the ABI encoding module.
    /// Encode is essentially a wrapper on top of `abi_encoder`,
    /// but it is responsible for parsing strings into proper `Token`s
    /// that can be encoded by the `abi_encoder`.
    pub fn encode(&self, abi: &str, fn_name: &str, values: &[&str]) -> Result<String, String> {
        let parsed_abi: JsonABI = serde_json::from_str(abi).unwrap();

        for entry in parsed_abi {
            if entry.name == fn_name {
                let raw_selector = self.build_fn_selector(fn_name, &entry.inputs);

                let mut encoder = ABIEncoder::new(raw_selector.as_bytes());

                let params: Vec<_> = entry
                    .inputs
                    .iter()
                    .map(|param| self.parse_param(param).unwrap())
                    .zip(values.iter().map(|v| v as &str))
                    .collect();

                let tokens = self.parse_tokens(&params).unwrap();

                let encoded = encoder.encode(&tokens).unwrap();

                let selector = encoder.function_selector;

                let mut encoded_abi: Vec<u8> = Vec::new();

                encoded_abi.extend_from_slice(&selector);
                encoded_abi.extend(encoded);

                return Ok(hex::encode(encoded_abi));
            }
        }
        Err("wrong function name".into())
    }

    pub fn parse_tokens<'a>(&self, params: &'a [(ParamType, &str)]) -> Result<Vec<Token>, String> {
        params
            .iter()
            .map(|&(ref param, value)| self.tokenize(param, value.to_string()))
            .collect::<Result<_, _>>()
            .map_err(From::from)
    }

    pub fn tokenize<'a>(&self, param: &ParamType, value: String) -> Result<Token, String> {
        let trimmed_value = value.trim();
        match &*param {
            ParamType::U8 => Ok(Token::U8(trimmed_value.parse::<u8>().unwrap())),
            ParamType::U16 => Ok(Token::U16(trimmed_value.parse::<u16>().unwrap())),
            ParamType::U32 => Ok(Token::U32(trimmed_value.parse::<u32>().unwrap())),
            ParamType::U64 => Ok(Token::U64(trimmed_value.parse::<u64>().unwrap())),
            ParamType::Bool => Ok(Token::Bool(trimmed_value.parse::<bool>().unwrap())),
            ParamType::Byte => Ok(Token::Byte(trimmed_value.parse::<u8>().unwrap())),
            ParamType::B256 => {
                let v = Vec::from_hex(trimmed_value).expect("invalid hex string");
                let s: [u8; 32] = v.as_slice().try_into().unwrap();
                Ok(Token::B256(s))
            }
            ParamType::Array(t, _) => Ok(self.tokenize_array(trimmed_value, &*t).unwrap()),
            ParamType::String(_) => Ok(Token::String(trimmed_value.to_string())),
            ParamType::Struct(struct_params) => {
                Ok(self.tokenize_struct(trimmed_value, struct_params).unwrap())
            }
            ParamType::Enum(s) => {
                let discriminant = self.get_enum_discriminant_from_string(&value);
                let value = self.get_enum_value_from_string(&value);

                let token = self.tokenize(&s[discriminant], value.to_owned())?;

                Ok(Token::Enum(Box::new((discriminant as u8, token))))
            }
        }
    }

    pub fn tokenize_struct(&self, value: &str, params: &[ParamType]) -> Result<Token, String> {
        if !value.starts_with('(') || !value.ends_with(')') {
            return Err("invalid data 1".into());
        }

        if value.chars().count() == 2 {
            return Ok(Token::Struct(vec![]));
        }

        let mut result = vec![];
        let mut nested = 0isize;
        let mut ignore = false;
        let mut last_item = 1;
        let mut params_iter = params.iter();

        for (pos, ch) in value.chars().enumerate() {
            match ch {
                '(' if !ignore => {
                    nested += 1;
                }
                ')' if !ignore => {
                    nested -= 1;

                    match nested.cmp(&0) {
                        std::cmp::Ordering::Less => {
                            return Err("invalid data".into());
                        }
                        std::cmp::Ordering::Equal => {
                            let sub = &value[last_item..pos];

                            let token = self.tokenize(
                                params_iter.next().ok_or("invalid data 2")?,
                                sub.to_string(),
                            )?;
                            result.push(token);
                            last_item = pos + 1;
                        }
                        _ => {}
                    }
                }
                '"' => {
                    ignore = !ignore;
                }
                ',' if nested == 1 && !ignore => {
                    let sub = &value[last_item..pos];
                    // If we've encountered an array within a struct property
                    // keep iterating until we see the end of it "]".
                    if sub.contains("[") && !sub.contains("]") {
                        continue;
                    }

                    let token = self
                        .tokenize(params_iter.next().ok_or("invalid data 2")?, sub.to_string())?;
                    result.push(token);
                    last_item = pos + 1;
                }
                _ => (),
            }
        }

        if ignore {
            return Err("invalid data 3".into());
        }

        Ok(Token::Struct(result))
    }

    pub fn tokenize_array<'a>(&self, value: &'a str, param: &ParamType) -> Result<Token, String> {
        if !value.starts_with('[') || !value.ends_with(']') {
            return Err("invalid data 1".into());
        }

        if value.chars().count() == 2 {
            return Ok(Token::Array(vec![]));
        }

        let mut result = vec![];
        let mut nested = 0isize;
        let mut ignore = false;
        let mut last_item = 1;
        for (i, ch) in value.chars().enumerate() {
            match ch {
                '[' if !ignore => {
                    nested += 1;
                }
                ']' if !ignore => {
                    nested -= 1;

                    match nested.cmp(&0) {
                        std::cmp::Ordering::Less => {
                            return Err("invalid data 2".into());
                        }
                        std::cmp::Ordering::Equal => {
                            // Last element of this nest level; proceed to tokenize.
                            let sub = &value[last_item..i];
                            match self.is_array(sub) {
                                true => {
                                    let arr_param = ParamType::Array(
                                        Box::new(param.to_owned()),
                                        self.get_array_length_from_string(sub),
                                    );

                                    result.push(self.tokenize(&arr_param, sub.to_string())?);
                                }
                                false => {
                                    result.push(self.tokenize(param, sub.to_string())?);
                                }
                            }

                            last_item = i + 1;
                        }
                        _ => {}
                    }
                }
                '"' => {
                    ignore = !ignore;
                }
                ',' if nested == 1 && !ignore => {
                    let sub = &value[last_item..i];
                    match self.is_array(sub) {
                        true => {
                            let arr_param = ParamType::Array(
                                Box::new(param.to_owned()),
                                self.get_array_length_from_string(sub),
                            );

                            result.push(self.tokenize(&arr_param, sub.to_string())?);
                        }
                        false => {
                            result.push(self.tokenize(param, sub.to_string())?);
                        }
                    }
                    last_item = i + 1;
                }
                _ => (),
            }
        }

        if ignore {
            return Err("invalid data 3".into());
        }

        Ok(Token::Array(result))
    }

    pub fn is_array(&self, ele: &str) -> bool {
        ele.starts_with("[") && ele.ends_with("]")
    }

    pub fn get_enum_discriminant_from_string(&self, ele: &str) -> usize {
        let mut chars = ele.chars();
        chars.next(); // Remove "("
        chars.next_back(); // Remove ")"
        let v: Vec<_> = chars.as_str().split(",").collect();
        v[0].parse().unwrap()
    }

    pub fn get_enum_value_from_string(&self, ele: &str) -> String {
        let mut chars = ele.chars();
        chars.next(); // Remove "("
        chars.next_back(); // Remove ")"
        let v: Vec<_> = chars.as_str().split(",").collect();
        v[1].to_string()
    }

    pub fn get_array_length_from_string(&self, ele: &str) -> usize {
        let mut chars = ele.chars();
        chars.next();
        chars.next_back();
        let stripped: Vec<_> = chars.as_str().split(",").collect();
        stripped.len()
    }

    pub fn build_fn_selector(&self, fn_name: &str, params: &[Property]) -> String {
        let mut fn_selector = fn_name.clone().to_owned();
        let mut args = String::new();
        let mut types: Vec<&str> = Vec::new();

        for i in 0..params.len() {
            let mut arg = "$".to_owned();
            arg.push_str(i.to_string().as_str());
            if i + 1 < params.len() {
                arg.push_str(",");
            }
            args.push_str(&arg);
            types.push(&params[i].type_field);
        }

        let args = format!("({})", args);

        fn_selector.push_str(&template_replace(&args, &types));

        fn_selector
    }

    /// Decodes a value of a given ABI and a target function's output.
    /// Note that the `value` has to be a byte array, meaning that
    /// the caller must properly cast the "upper" type into a `&[u8]`,
    pub fn decode<'a>(
        &self,
        abi: &str,
        fn_name: &str,
        value: &'a [u8],
    ) -> Result<Vec<Token>, String> {
        let parsed_abi: JsonABI = serde_json::from_str(abi).unwrap();

        for entry in parsed_abi {
            if entry.name == fn_name {
                let params: Vec<_> = entry
                    .outputs
                    .iter()
                    .map(|param| self.parse_param(param).unwrap())
                    .collect();

                let mut decoder = ABIDecoder::new();

                let decoded = decoder.decode(&params, value).unwrap();

                return Ok(decoded);
            }
        }

        Err("wrong".into())
    }

    /// Turns a JSON property into ParamType
    pub fn parse_param(&self, param: &Property) -> Result<ParamType, String> {
        if param.type_field == "struct" {
            let mut params: Vec<ParamType> = vec![];
            let components = param.components.as_ref().unwrap();
            for component in components {
                params.push(self.parse_param(&component)?);
            }

            return Ok(ParamType::Struct(params));
        }
        if param.type_field == "enum" {
            let mut params: Vec<ParamType> = vec![];
            let components = param.components.as_ref().unwrap();
            for component in components {
                params.push(self.parse_param(&component)?);
            }

            return Ok(ParamType::Enum(params));
        }

        match param.type_field.contains("[") && param.type_field.contains("]") {
            // Simple case (u<M>, bool, etc.)
            false => Ok(ParamType::from_str(&param.type_field.clone()).unwrap()),
            // Either array (<T>[<M>]) or string (str[<M>])
            true => {
                let split: Vec<&str> = param.type_field.split("[").collect();
                if split.len() != 2 {
                    panic!("invalid data")
                }

                let param_type = ParamType::from_str(split[0]).unwrap();
                let size: usize = split[1][..split[1].len() - 1].parse().unwrap();

                if let ParamType::String(_) = param_type {
                    // String
                    Ok(ParamType::String(size))
                } else {
                    // Array
                    Ok(ParamType::Array(Box::new(param_type), size))
                }
            }
        }
    }
}

fn template_replace(template: &str, values: &[&str]) -> String {
    let regex = Regex::new(r#"\$(\d+)"#).unwrap();
    regex
        .replace_all(template, |captures: &Captures| {
            values.get(index(captures)).unwrap_or(&"")
        })
        .to_string()
}

fn index(captures: &Captures) -> usize {
    captures.get(1).unwrap().as_str().parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"u32"
                    }
                ],
                "name":"takes_u32_returns_bool",
                "outputs":[
                    {
                        "name":"",
                        "type":"bool"
                    }
                ]
            }
        ]
        "#;

        let values = vec!["10"];

        let abi = ABI::new();

        let function_name = "takes_u32_returns_bool";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode = "000000006355e6ee000000000000000a";
        assert_eq!(encoded, expected_encode);

        let return_value = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // false
        ];

        let decoded_return = abi.decode(json_abi, function_name, &return_value).unwrap();

        let expected_return = vec![Token::Bool(false)];

        assert_eq!(decoded_return, expected_return);
    }

    #[test]
    fn b256_and_single_byte_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"foo",
                        "type":"b256"
                    },
                    {
                        "name":"bar",
                        "type":"byte"
                    }
                ],
                "name":"my_func",
                "outputs":[
                    {
                        "name":"",
                        "type":"b256"
                    }
                ]
            }
        ]
        "#;

        let values = vec![
            "d5579c46dfcc7f18207013e65b44e4cb4e2c2298f4ac457ba8f82743f31e930b",
            "1",
        ];

        let abi = ABI::new();

        let function_name = "my_func";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode = "00000000e64019abd5579c46dfcc7f18207013e65b44e4cb4e2c2298f4ac457ba8f82743f31e930b0000000000000001";
        assert_eq!(encoded, expected_encode);

        let return_value =
            hex::decode("a441b15fe9a3cf56661190a0b93b9dec7d04127288cc87250967cf3b52894d11")
                .unwrap();

        let decoded_return = abi.decode(json_abi, function_name, &return_value).unwrap();

        let s: [u8; 32] = return_value.as_slice().try_into().unwrap();
        let b256 = s.try_into().unwrap();

        let expected_return = vec![Token::B256(b256)];

        assert_eq!(decoded_return, expected_return);
    }

    #[test]
    fn array_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"u16[3]"
                    }
                ],
                "name":"takes_array",
                "outputs":[
                    {
                        "name":"",
                        "type":"u16[2]"
                    }
                ]
            }
        ]
        "#;

        let values = vec!["[1,2,3]"];

        let abi = ABI::new();

        let function_name = "takes_array";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode = "00000000f0b87864000000000000000100000000000000020000000000000003";
        assert_eq!(encoded, expected_encode);

        let return_value = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // 0
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, // 1
        ];

        let decoded_return = abi.decode(json_abi, function_name, &return_value).unwrap();

        let expected_return = vec![Token::Array(vec![Token::U16(0), Token::U16(1)])];

        assert_eq!(decoded_return, expected_return);
    }

    #[test]
    fn tokenize_array() {
        let abi = ABI::new();

        let value = "[[1,2],[3],4]";
        let param = ParamType::U16;
        let tokens = abi.tokenize_array(value, &param).unwrap();

        let expected_tokens = Token::Array(vec![
            Token::Array(vec![Token::U16(1), Token::U16(2)]), // First element, a sub-array with 2 elements
            Token::Array(vec![Token::U16(3)]), // Second element, a sub-array with 1 element
            Token::U16(4),                     // Third element
        ]);

        assert_eq!(tokens, expected_tokens);

        let value = "[1,[2],[3],[4,5]]";
        let param = ParamType::U16;
        let tokens = abi.tokenize_array(value, &param).unwrap();

        let expected_tokens = Token::Array(vec![
            Token::U16(1),
            Token::Array(vec![Token::U16(2)]),
            Token::Array(vec![Token::U16(3)]),
            Token::Array(vec![Token::U16(4), Token::U16(5)]),
        ]);

        assert_eq!(tokens, expected_tokens);

        let value = "[1,2,3,4,5]";
        let param = ParamType::U16;
        let tokens = abi.tokenize_array(value, &param).unwrap();

        let expected_tokens = Token::Array(vec![
            Token::U16(1),
            Token::U16(2),
            Token::U16(3),
            Token::U16(4),
            Token::U16(5),
        ]);

        assert_eq!(tokens, expected_tokens);

        let value = "[[1,2,3,[4,5]]]";
        let param = ParamType::U16;
        let tokens = abi.tokenize_array(value, &param).unwrap();

        let expected_tokens = Token::Array(vec![Token::Array(vec![
            Token::U16(1),
            Token::U16(2),
            Token::U16(3),
            Token::Array(vec![Token::U16(4), Token::U16(5)]),
        ])]);

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn nested_array_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"u16[3]"
                    }
                ],
                "name":"takes_nested_array",
                "outputs":[
                    {
                        "name":"",
                        "type":"u16[2]"
                    }
                ]
            }
        ]
        "#;

        let values = vec!["[[1,2],[3],[4]]"];

        let abi = ABI::new();

        let function_name = "takes_nested_array";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode =
            "00000000e5d521030000000000000001000000000000000200000000000000030000000000000004";
        assert_eq!(encoded, expected_encode);

        let return_value = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // 0
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, // 1
        ];

        let decoded_return = abi.decode(json_abi, function_name, &return_value).unwrap();

        let expected_return = vec![Token::Array(vec![Token::U16(0), Token::U16(1)])];

        assert_eq!(decoded_return, expected_return);
    }

    #[test]
    fn string_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"foo",
                        "type":"str[23]"
                    }
                ],
                "name":"takes_string",
                "outputs":[
                    {
                        "name":"",
                        "type":"str[2]"
                    }
                ]
            }
        ]
        "#;

        let values = vec!["This is a full sentence"];

        let abi = ABI::new();

        let function_name = "takes_string";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode = "00000000d56e76515468697320697320612066756c6c2073656e74656e636500";
        assert_eq!(encoded, expected_encode);

        let return_value = [
            0x4f, 0x4b, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // "OK" encoded in utf8
        ];

        let decoded_return = abi.decode(json_abi, function_name, &return_value).unwrap();

        let expected_return = vec![Token::String("OK".into())];

        assert_eq!(decoded_return, expected_return);
    }

    #[test]
    fn struct_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyStruct",
                        "type":"struct",
                        "components": [
                            {
                                "name": "foo",
                                "type": "u8"
                            },
                            {
                                "name": "bar",
                                "type": "bool"
                            }
                        ]
                    }
                ],
                "name":"takes_struct",
                "outputs":[]
            }
        ]
        "#;

        let values = vec!["(42, true)"];

        let abi = ABI::new();

        let function_name = "takes_struct";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode = "00000000f5957fce000000000000002a0000000000000001";
        assert_eq!(encoded, expected_encode);
    }

    #[test]
    fn nested_struct_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyNestedStruct",
                        "type":"struct",
                        "components": [
                            {
                                "name": "x",
                                "type": "u16"
                            },
                            {
                                "name": "y",
                                "type": "struct",
                                "components": [
                                    {
                                        "name":"a",
                                        "type": "bool"
                                    },
                                    {
                                        "name":"b",
                                        "type": "u8[2]"
                                    }
                                ]
                            }
                        ]
                    }
                ],
                "name":"takes_nested_struct",
                "outputs":[]
            }
        ]
        "#;

        let values = vec!["(10, (true, [1,2]))"];

        let abi = ABI::new();

        let function_name = "takes_nested_struct";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode =
            "00000000e8a04d9c000000000000000a000000000000000100000000000000010000000000000002";
        assert_eq!(encoded, expected_encode);

        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyNestedStruct",
                        "type":"struct",
                        "components": [
                            {
                                "name": "x",
                                "type": "struct",
                                "components": [
                                    {
                                        "name":"a",
                                        "type": "bool"
                                    },
                                    {
                                        "name":"b",
                                        "type": "u8[2]"
                                    }
                                ]
                            },
                            {
                                "name": "y",
                                "type": "u16"
                            }
                        ]
                    }
                ],
                "name":"takes_nested_struct",
                "outputs":[]
            }
        ]
        "#;

        let values = vec!["((true, [1,2]), 10)"];

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode =
            "00000000e8a04d9c000000000000000100000000000000010000000000000002000000000000000a";
        assert_eq!(encoded, expected_encode);
    }

    #[test]
    fn enum_encode_and_decode() {
        let json_abi = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyEnum",
                        "type":"enum",
                        "components": [
                            {
                                "name": "x",
                                "type": "u32"
                            },
                            {
                                "name": "y",
                                "type": "bool"
                            }
                        ]
                    }
                ],
                "name":"takes_enum",
                "outputs":[]
            }
        ]
        "#;

        let values = vec!["(0, 42)"];

        let abi = ABI::new();

        let function_name = "takes_enum";

        let encoded = abi.encode(json_abi, function_name, &values).unwrap();
        println!("encoded: {:?}\n", encoded);

        let expected_encode = "000000009542a3c90000000000000000000000000000002a";
        assert_eq!(encoded, expected_encode);
    }
}
