use crate::util::encode::{Token, Type};
use fuel_abi_types::abi::full_program::FullProgramABI;
use fuels_core::codec::{ABIEncoder, EncoderConfig};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ScriptCallHandler {
    main_arg_types: Vec<Type>,
}

impl ScriptCallHandler {
    const MAIN_KEYWORD: &'static str = "main";

    /// Generate a new call handler for calling script main function from the json abi.
    ///
    /// Proviede json abi is used for determining the argument types, this is required as the data
    /// encoding is requires the type of the data.
    pub(crate) fn from_json_abi_str(json_abi_str: &str) -> anyhow::Result<Self> {
        let full_abi = FullProgramABI::from_json_abi(json_abi_str)?;
        // Note: using .expect() here is safe since a script without a main function is a compile
        // error and the fact that we have the json abi of the built script suggests that this is a
        // valid script.
        let main_function = full_abi
            .functions
            .iter()
            .find(|abi_func| abi_func.name() == Self::MAIN_KEYWORD)
            .expect("every valid script needs to have a main function");
        let main_arg_types = main_function
            .inputs()
            .iter()
            .map(Type::try_from)
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self { main_arg_types })
    }

    /// Encode the provided values with script's main argument types.
    ///
    /// Returns an error if the provided value count does not match the number of arguments.
    pub(crate) fn encode_arguments(&self, values: &[&str]) -> anyhow::Result<Vec<u8>> {
        let main_arg_types = &self.main_arg_types;
        let expected_arg_count = main_arg_types.len();
        let provided_arg_count = values.len();

        if expected_arg_count != provided_arg_count {
            anyhow::bail!(
                "main function takes {expected_arg_count} arguments, {provided_arg_count} provided"
            );
        }

        let tokens = main_arg_types
            .iter()
            .zip(values.iter())
            .map(|(ty, val)| Token::from_type_and_value(ty, val).map(|token| token.0))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let abi_encoder = ABIEncoder::new(EncoderConfig::default());
        Ok(abi_encoder.encode(tokens.as_slice())?)
    }
}

#[cfg(test)]
mod tests {
    use super::{ScriptCallHandler, Type};

    #[test]
    fn test_script_call_handler_generation_success() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let generated_call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();

        let expected_call_handler = ScriptCallHandler {
            main_arg_types: vec![Type::U8, Type::Bool],
        };

        assert_eq!(generated_call_handler, expected_call_handler);
    }

    #[test]
    #[should_panic]
    fn test_script_call_handler_generation_fail_missing_main() {
        let test_json_abi =
            r#"{"types":[],"functions":[],"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
    }

    #[test]
    fn test_main_encoding_success() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
        let values = ["2", "true"];

        let encoded_bytes = call_handler.encode_arguments(&values).unwrap();
        let expected_bytes = vec![2u8, 1u8];
        assert_eq!(encoded_bytes, expected_bytes);
    }

    #[test]
    #[should_panic]
    fn test_main_encoding_fail_arg_type_mismatch() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
        // The abi describes the following main function:
        // - fn main(test_u8: u8, test_bool: bool)
        // Providing a bool to u8 field should return an error.
        let values = ["true", "2"];
        call_handler.encode_arguments(&values).unwrap();
    }

    #[test]
    #[should_panic(expected = "main function takes 2 arguments, 1 provided")]
    fn test_main_encoding_fail_arg_count_mismatch() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
        // The abi describes the following main function:
        // - fn main(test_u8: u8, test_bool: bool)
        // Providing only 1 value should return an error as function requires 2 args.
        let values = ["true"];
        call_handler.encode_arguments(&values).unwrap();
    }
}
