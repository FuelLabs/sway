use crate::op::call::{
    parser::{get_default_value, param_to_function_arg, param_type_val_to_token, token_to_string},
    Abi,
};
use anyhow::{anyhow, Result};
use fuels_core::types::{param_types::ParamType, ContractId};
use std::collections::HashMap;
use std::io::Write;

/// List all functions in the contracts' ABIs along with examples of how to call them.
/// This function supports listing functions from multiple contracts when additional
/// contract ABIs are provided via the --contract-abi parameter.
pub fn list_contract_functions<W: Write>(
    main_contract_id: &ContractId,
    abi_map: &HashMap<ContractId, Abi>,
    writer: &mut W,
) -> Result<()> {
    // First, list functions for the main contract
    if let Some(main_abi) = abi_map.get(main_contract_id) {
        list_functions_for_single_contract(main_contract_id, main_abi, true, writer)?;
    } else {
        return Err(anyhow!("Main contract ABI not found in abi_map"));
    }

    // Then, list functions for additional contracts if any
    let additional_contracts: Vec<_> = abi_map
        .iter()
        .filter(|(id, _)| *id != main_contract_id)
        .collect();

    if !additional_contracts.is_empty() {
        writeln!(writer, "\n{}", "=".repeat(80))?;
        writeln!(writer, "Additional Contracts:\n")?;

        for (contract_id, abi) in additional_contracts {
            list_functions_for_single_contract(contract_id, abi, false, writer)?;
        }
    }

    Ok(())
}

/// List functions for a single contract
fn list_functions_for_single_contract<W: Write>(
    contract_id: &ContractId,
    abi: &Abi,
    is_main_contract: bool,
    writer: &mut W,
) -> Result<()> {
    let header = if is_main_contract {
        format!("Callable functions for contract: {}\n", contract_id)
    } else {
        format!("Functions for additional contract: {}\n", contract_id)
    };

    writeln!(writer, "{}", header)?;

    if abi.unified.functions.is_empty() {
        writeln!(writer, "No functions found in the contract ABI.\n")?;
        return Ok(());
    }

    for func in &abi.unified.functions {
        let func_args = func
            .inputs
            .iter()
            .map(|input| {
                let Ok(param_type) = ParamType::try_from_type_application(input, &abi.type_lookup)
                else {
                    return Err(anyhow!("Failed to convert input type application"));
                };
                let func_args = format!("{}: {}", input.name, param_to_function_arg(&param_type));
                let func_args_input = {
                    let token =
                        param_type_val_to_token(&param_type, &get_default_value(&param_type))
                            .map_err(|err| {
                                anyhow!(
                                    "Failed to generate example call for {}: {}",
                                    func.name,
                                    err
                                )
                            })?;
                    token_to_string(&token).map_err(|err| {
                        anyhow!(
                            "Failed to convert token to string for {}: {}",
                            func.name,
                            err
                        )
                    })?
                };
                Ok((func_args, func_args_input, param_type))
            })
            .collect::<Result<Vec<_>>>()?;

        let func_args_types = func_args
            .iter()
            .map(|(func_args, _, _)| func_args.to_owned())
            .collect::<Vec<String>>()
            .join(", ");

        let func_args_inputs = func_args
            .iter()
            .map(|(_, func_args_input, param_type)| match param_type {
                ParamType::Array(_, _)
                | ParamType::Unit
                | ParamType::Tuple(_)
                | ParamType::Struct { .. }
                | ParamType::Enum { .. }
                | ParamType::RawSlice
                | ParamType::Vector(_) => format!("\"{}\"", func_args_input),
                _ => func_args_input.to_owned(),
            })
            .collect::<Vec<String>>()
            .join(" ");

        let return_type = ParamType::try_from_type_application(&func.output, &abi.type_lookup)
            .map(|param_type| param_to_function_arg(&param_type))
            .map_err(|err| {
                anyhow!(
                    "Failed to convert output type application for {}: {}",
                    func.name,
                    err
                )
            })?;

        let painted_name = forc_util::ansiterm::Colour::Blue.paint(func.name.clone());
        writeln!(
            writer,
            "{}({}) -> {}",
            painted_name, func_args_types, return_type
        )?;
        writeln!(
            writer,
            "  forc call \\\n      --abi {} \\\n      {} \\\n      {} {}\n",
            abi.source, contract_id, func.name, func_args_inputs,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op::call::tests::get_contract_instance;
    use std::{io::Cursor, path::Path, str::FromStr};

    #[tokio::test]
    async fn test_list_contract_functions_preserves_abi_source_format() {
        let (_, id, _, _) = get_contract_instance().await;

        // Load a test ABI content
        let abi_path_str = "../../forc-plugins/forc-client/test/data/contract_with_types/contract_with_types-abi.json";
        let abi_path = Path::new(abi_path_str);
        let abi_str = std::fs::read_to_string(abi_path).unwrap();

        // Test different source formats
        let test_cases = vec![
            (
                "file_path",
                crate::cmd::call::AbiSource::File(std::path::PathBuf::from("./test-abi.json")),
                "--abi ./test-abi.json",
            ),
            (
                "url",
                crate::cmd::call::AbiSource::Url(
                    url::Url::parse("https://example.com/abi.json").unwrap(),
                ),
                "--abi https://example.com/abi.json",
            ),
            (
                "inline_json",
                crate::cmd::call::AbiSource::String(r#"{"test":"value"}"#.to_string()),
                r#"--abi {"test":"value"}"#,
            ),
        ];

        for (test_name, source, expected_abi_arg) in test_cases {
            let abi = Abi::from_str(&abi_str).unwrap().with_source(source);
            let mut abi_map = HashMap::new();
            abi_map.insert(id, abi);

            let mut output = Cursor::new(Vec::<u8>::new());
            list_contract_functions(&id, &abi_map, &mut output).unwrap();

            let output_bytes = output.into_inner();
            let output_string = String::from_utf8(output_bytes).unwrap();

            // Verify the ABI source is preserved exactly as provided
            assert!(
                output_string.contains(expected_abi_arg),
                "Test '{}' failed: expected '{}' in output, but got:\n{}",
                test_name,
                expected_abi_arg,
                output_string
            );
        }
    }
}
