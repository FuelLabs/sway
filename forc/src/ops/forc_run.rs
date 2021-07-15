use std::path::PathBuf;

use core_lang::parse;
use fuel_tx::Transaction;
use tx_client::client::TxClient;

use crate::cli::{BuildCommand, RunCommand};
use crate::ops::forc_build;
use crate::utils::cli_error::CliError;
use crate::utils::{constants, helpers};
use constants::{DEFAULT_NODE_URL, SWAY_CONTRACT, SWAY_LIBRARY, SWAY_PREDICATE, SWAY_SCRIPT};
use helpers::{find_manifest_dir, get_main_file, read_manifest};

pub async fn run(command: RunCommand) -> Result<(), CliError> {
    let path_dir = PathBuf::from(&command.path);

    match find_manifest_dir(&path_dir) {
        Some(manifest_dir) => {
            let manifest = read_manifest(&manifest_dir)?;
            let project_name = &manifest.project.name;
            let main_file = get_main_file(&manifest, &manifest_dir)?;

            // parse the main file and check is it a script
            match parse(main_file) {
                core_lang::CompileResult::Ok {
                    value: parse_tree,
                    warnings: _,
                    errors: _,
                } => {
                    if let Some(_) = &parse_tree.script_ast {
                        let input_data = &command.data.unwrap_or("".into());
                        let data = format_hex_data(input_data);
                        let script_data = hex::decode(data).expect("Invalid hex");

                        let build_command = BuildCommand {
                            path: Some(command.path),
                            print_asm: false,
                            binary_outfile: None,
                            offline_mode: false,
                        };

                        let compiled_script = forc_build::build(build_command)?;
                        let tx = create_tx_with_script_and_data(compiled_script, script_data);

                        if command.dry_run {
                            println!("{:?}", tx);
                        } else {
                            let node_url = match &manifest.network {
                                Some(network) => &network.url,
                                _ => DEFAULT_NODE_URL,
                            };

                            let client = TxClient::new(node_url)?;

                            match client.transact(&tx).await {
                                Ok(logs) => {
                                    println!("{:?}", logs);
                                }
                                Err(e) => return Err(e.to_string().into()),
                            }
                        }

                        Ok(())
                    } else {
                        let parse_type = {
                            if parse_tree.contract_ast.is_some() {
                                SWAY_CONTRACT
                            } else if parse_tree.predicate_ast.is_some() {
                                SWAY_PREDICATE
                            } else {
                                SWAY_LIBRARY
                            }
                        };

                        Err(CliError::wrong_sway_type(
                            project_name,
                            SWAY_SCRIPT,
                            parse_type,
                        ))
                    }
                }
                core_lang::CompileResult::Err {
                    warnings: _,
                    errors,
                } => Err(CliError::parsing_failed(project_name, errors)),
            }
        }
        None => Err(CliError::manifest_file_missing(path_dir)),
    }
}

fn create_tx_with_script_and_data(script: Vec<u8>, script_data: Vec<u8>) -> Transaction {
    let gas_price = 0;
    let gas_limit = 10000000;
    let maturity = 0;
    let inputs = vec![];
    let outputs = vec![];
    let witnesses = vec![];

    Transaction::script(
        gas_price,
        gas_limit,
        maturity,
        script,
        script_data,
        inputs,
        outputs,
        witnesses,
    )
}

// cut '0x' from the start
fn format_hex_data(data: &str) -> &str {
    if data.len() >= 2 && &data[..2] == "0x" {
        &data[2..]
    } else {
        &data
    }
}
