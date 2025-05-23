use crate::op::call::parser::{
    get_default_value, param_to_function_arg, param_type_val_to_token, token_to_string,
};
use anyhow::{anyhow, Result};
use either::Either;
use fuel_abi_types::abi::unified_program::UnifiedProgramABI;
use fuels_core::types::{param_types::ParamType, ContractId};
use std::collections::HashMap;
use std::io::Write;

/// List all functions in a contract's ABI along with examples of how to call them.
pub fn list_contract_functions<W: Write>(
    contract_id: &ContractId,
    abi: &Either<std::path::PathBuf, reqwest::Url>,
    unified_program_abi: &UnifiedProgramABI,
    writer: &mut W,
) -> Result<()> {
    writeln!(
        writer,
        "\nCallable functions for contract: {}\n",
        contract_id
    )?;

    if unified_program_abi.functions.is_empty() {
        writeln!(writer, "No functions found in the contract ABI.")?;
        return Ok(());
    }

    let type_lookup = unified_program_abi
        .types
        .iter()
        .map(|decl| (decl.type_id, decl.clone()))
        .collect::<HashMap<_, _>>();

    for func in &unified_program_abi.functions {
        let func_args = func
            .inputs
            .iter()
            .map(|input| {
                let Ok(param_type) = ParamType::try_from_type_application(input, &type_lookup)
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

        let return_type = ParamType::try_from_type_application(&func.output, &type_lookup)
            .map(|param_type| param_to_function_arg(&param_type))
            .map_err(|err| {
                anyhow!(
                    "Failed to convert output type application for {}: {}",
                    func.name,
                    err
                )
            })?;

        // Get the ABI path or URL as a string
        let raw_abi_input = match abi {
            Either::Left(path) => path.to_str().unwrap_or("").to_owned(),
            Either::Right(url) => url.to_string(),
        };

        let painted_name = forc_util::ansiterm::Colour::Blue.paint(func.name.clone());
        writeln!(
            writer,
            "{}({}) -> {}",
            painted_name, func_args_types, return_type
        )?;
        writeln!(
            writer,
            "  forc call \\\n      --abi {} \\\n      {} \\\n      {} {}\n",
            raw_abi_input, contract_id, func.name, func_args_inputs,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op::call::tests::get_contract_instance;
    use fuel_abi_types::abi::program::ProgramABI;
    use std::io::Cursor;
    use std::path::Path;

    #[tokio::test]
    async fn test_list_contract_functions() {
        let (_, id, _, _) = get_contract_instance().await;

        // Load a test ABI
        let abi_path_str = "../../forc-plugins/forc-client/tests/data/contract_with_types/contract_with_types-abi.json";
        let abi_path = Path::new(abi_path_str);
        let abi = Either::Left(abi_path.to_path_buf());

        let abi_str = std::fs::read_to_string(abi_path).unwrap();
        let parsed_abi: ProgramABI = serde_json::from_str(&abi_str).unwrap();
        let unified_program_abi = UnifiedProgramABI::from_counterpart(&parsed_abi).unwrap();

        // Use a buffer to capture the output
        let mut output = Cursor::new(Vec::<u8>::new());

        // Call function with our buffer as the writer
        list_contract_functions(&id, &abi, &unified_program_abi, &mut output)
            .expect("Failed to list contract functions");

        // Get the output as a string
        let output_bytes = output.into_inner();
        let output_string = String::from_utf8(output_bytes).expect("Output was not valid UTF-8");

        // Verify the output contains expected function names and formatting
        assert!(output_string.contains("Callable functions for contract:"));

        assert!(output_string.contains(
            "\u{1b}[34mtest_struct_with_generic\u{1b}[0m(a: GenericStruct) -> GenericStruct"
        ));
        assert!(output_string.contains("forc call \\"));
        assert!(output_string.contains(format!("--abi {abi_path_str} \\").as_str()));
        assert!(output_string.contains(format!("{id} \\").as_str()));
        assert!(output_string.contains("test_struct_with_generic \"{0, aaaa}\""));

        assert!(output_string
            .contains("\u{1b}[34mtest_complex_struct\u{1b}[0m(a: ComplexStruct) -> ComplexStruct"));
        assert!(output_string.contains("forc call \\"));
        assert!(output_string.contains(format!("--abi {abi_path_str} \\").as_str()));
        assert!(output_string.contains(format!("{id} \\").as_str()));
        assert!(output_string.contains(
            "test_complex_struct \"{({aa, 0}, 0), (Active:false), 0, {{0, aaaa}, aaaa}}\""
        ));
    }
}
