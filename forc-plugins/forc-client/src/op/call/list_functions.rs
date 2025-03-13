use crate::op::call::parser::{
    get_default_value, param_to_function_arg, param_type_val_to_token, token_to_string,
};
use anyhow::{anyhow, bail, Result};
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
        "\nAvailable functions in contract: {}\n",
        contract_id
    )?;

    if unified_program_abi.functions.is_empty() {
        writeln!(writer, "No functions found in the contract ABI.")?;
    } else {
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
                    let func_args =
                        format!("{}: {}", input.name, param_to_function_arg(&param_type));
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
                    Ok((func_args, func_args_input))
                })
                .collect::<Result<Vec<_>>>()?;
            let func_args_types = func_args
                .iter()
                .map(|(func_args, _)| func_args.to_owned())
                .collect::<Vec<String>>()
                .join(", ");
            let func_args_inputs = func_args
                .iter()
                .map(|(_, func_args_input)| func_args_input.to_owned())
                .collect::<Vec<String>>()
                .join(", ");

            let return_type = match ParamType::try_from_type_application(&func.output, &type_lookup)
            {
                Ok(param_type) => param_to_function_arg(&param_type),
                Err(err) => bail!(
                    "Failed to convert output type application for {}: {}",
                    func.name,
                    err
                ),
            };

            // Get the ABI path or URL as a string
            let raw_abi_input = match abi {
                Either::Left(path) => path.to_str().unwrap_or("").to_owned(),
                Either::Right(url) => url.to_string(),
            };

            writeln!(
                writer,
                "{}({}) -> {}",
                func.name, func_args_types, return_type
            )?;
            let args_part = if func_args_inputs.is_empty() {
                String::new()
            } else {
                format!("\"{}\"", func_args_inputs)
            };
            writeln!(
                writer,
                "  forc call \\\n      --abi {} \\\n      {} \\\n      {} {}\n",
                raw_abi_input, contract_id, func.name, args_part,
            )?;
        }
    }

    Ok(())
}
