use core_lang::parse;
use fuel_client::client::FuelClient;
use fuel_tx::Transaction;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::process::Child;

use crate::cli::{BuildCommand, RunCommand};
use crate::ops::forc_build;
use crate::utils::cli_error::CliError;
use crate::utils::client::start_fuel_core;

use crate::utils::{constants, helpers};
use constants::{SWAY_CONTRACT, SWAY_LIBRARY, SWAY_PREDICATE, SWAY_SCRIPT};
use helpers::{find_manifest_dir, get_main_file, read_manifest};

pub async fn run(command: RunCommand) -> Result<(), CliError> {
    let path_dir = if let Some(path) = &command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().unwrap()
    };

    match find_manifest_dir(&path_dir) {
        Some(manifest_dir) => {
            let manifest = read_manifest(&manifest_dir)?;
            let project_name = &manifest.project.name;
            let main_file = get_main_file(&manifest, &manifest_dir)?;

            // parse the main file and check is it a script
            let parsed_result = parse(main_file, None);
            match parsed_result.value {
                Some(parse_tree) => {
                    if let Some(_) = &parse_tree.script_ast {
                        let input_data = &command.data.unwrap_or("".into());
                        let data = format_hex_data(input_data);
                        let script_data = hex::decode(data).expect("Invalid hex");

                        let build_command = BuildCommand {
                            path: command.path,
                            print_finalized_asm: command.print_finalized_asm,
                            print_intermediate_asm: command.print_intermediate_asm,
                            binary_outfile: command.binary_outfile,
                            offline_mode: false,
                            silent_mode: command.silent_mode,
                        };

                        let compiled_script = forc_build::build(build_command)?;
                        let (inputs, outputs) = manifest
                            .get_tx_inputs_and_outputs()
                            .map_err(|message| CliError { message })?;

                        let tx = create_tx_with_script_and_data(
                            compiled_script,
                            script_data,
                            inputs,
                            outputs,
                        );

                        if command.dry_run {
                            println!("{:?}", tx);
                            Ok(())
                        } else {
                            let node_url = match &manifest.network {
                                Some(network) => &network.url,
                                _ => &command.node_url,
                            };

                            let child = try_send_tx(node_url, &tx, command.pretty_print).await?;

                            if command.kill_node {
                                if let Some(mut child) = child {
                                    child.kill().await.expect("Node should be killed");
                                }
                            }

                            Ok(())
                        }
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
                None => Err(CliError::parsing_failed(project_name, parsed_result.errors)),
            }
        }
        None => Err(CliError::manifest_file_missing(path_dir)),
    }
}

async fn try_send_tx(
    node_url: &str,
    tx: &Transaction,
    pretty_print: bool,
) -> Result<Option<Child>, CliError> {
    let client = FuelClient::new(node_url)?;

    match client.health().await {
        Ok(_) => {
            send_tx(&client, tx, pretty_print).await?;
            Ok(None)
        }
        Err(_) => {
            print!(
                "We noticed you don't have fuel-core running, would you like to start a node [y/n]?"
            );
            io::stdout().flush().unwrap();
            let mut reply = String::new();
            io::stdin().read_line(&mut reply)?;
            let reply = reply.trim().to_lowercase();

            if reply == "y" || reply == "yes" {
                let child = start_fuel_core(node_url, &client).await?;
                send_tx(&client, tx, pretty_print).await?;
                Ok(Some(child))
            } else {
                Ok(None)
            }
        }
    }
}

async fn send_tx(
    client: &FuelClient,
    tx: &Transaction,
    pretty_print: bool,
) -> Result<(), CliError> {
    match client.transact(&tx).await {
        Ok(logs) => {
            if pretty_print {
                println!("{:#?}", logs);
            } else {
                println!("{:?}", logs);
            }
            Ok(())
        }
        Err(e) => Err(e.to_string().into()),
    }
}

fn create_tx_with_script_and_data(
    script: Vec<u8>,
    script_data: Vec<u8>,
    inputs: Vec<fuel_tx::Input>,
    outputs: Vec<fuel_tx::Output>,
) -> Transaction {
    let gas_price = 0;
    let gas_limit = 10000000;
    let maturity = 0;
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
