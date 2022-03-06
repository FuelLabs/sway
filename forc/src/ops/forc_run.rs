use fuel_gql_client::client::FuelClient;
use fuel_tx::Transaction;
use futures::TryFutureExt;

use std::path::PathBuf;
use std::str::FromStr;
use sway_core::{parse, TreeType};
use tokio::process::Child;

use crate::cli::{BuildCommand, RunCommand};
use crate::ops::forc_build;
use crate::utils::cli_error::CliError;

use crate::utils::helpers;
use helpers::{get_main_file, read_manifest};
use sway_utils::{constants::*, find_manifest_dir};

pub async fn run(command: RunCommand) -> Result<(), CliError> {
    let path_dir = if let Some(path) = &command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| format!("{:?}", e))?
    };

    match find_manifest_dir(&path_dir) {
        Some(manifest_dir) => {
            let manifest = read_manifest(&manifest_dir)?;
            let project_name = &manifest.project.name;
            let main_file = get_main_file(&manifest, &manifest_dir)?;

            // parse the main file and check is it a script
            let parsed_result = parse(main_file, None);
            match parsed_result.value {
                Some(parse_tree) => match parse_tree.tree_type {
                    TreeType::Script => {
                        let input_data = &command.data.unwrap_or_else(|| "".into());
                        let data = format_hex_data(input_data);
                        let script_data = hex::decode(data).expect("Invalid hex");

                        let build_command = BuildCommand {
                            path: command.path,
                            use_ir: command.use_ir,
                            print_finalized_asm: command.print_finalized_asm,
                            print_intermediate_asm: command.print_intermediate_asm,
                            print_ir: command.print_ir,
                            binary_outfile: command.binary_outfile,
                            debug_outfile: command.debug_outfile,
                            offline_mode: false,
                            silent_mode: command.silent_mode,
                            output_directory: command.output_directory,
                            minify_json_abi: command.minify_json_abi,
                        };

                        let compiled = forc_build::build(build_command)?;
                        let contracts = command.contract.unwrap_or_default();
                        let (inputs, outputs) = get_tx_inputs_and_outputs(contracts);

                        let tx = create_tx_with_script_and_data(
                            compiled.bytecode,
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
                    }
                    TreeType::Contract => Err(CliError::wrong_sway_type(
                        project_name,
                        SWAY_SCRIPT,
                        SWAY_CONTRACT,
                    )),
                    TreeType::Predicate => Err(CliError::wrong_sway_type(
                        project_name,
                        SWAY_SCRIPT,
                        SWAY_PREDICATE,
                    )),
                    TreeType::Library { .. } => Err(CliError::wrong_sway_type(
                        project_name,
                        SWAY_SCRIPT,
                        SWAY_LIBRARY,
                    )),
                },
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
        Err(_) => Err(CliError::fuel_core_not_running(node_url)),
    }
}

async fn send_tx(
    client: &FuelClient,
    tx: &Transaction,
    pretty_print: bool,
) -> Result<(), CliError> {
    let id = format!("{:#x}", tx.id());
    match client
        .submit(tx)
        .and_then(|_| client.receipts(id.as_str()))
        .await
    {
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
    let gas_limit = fuel_tx::consts::MAX_GAS_PER_TX;
    let byte_price = 0;
    let maturity = 0;
    let witnesses = vec![];

    Transaction::script(
        gas_price,
        gas_limit,
        byte_price,
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
    data.strip_prefix("0x").unwrap_or(data)
}

fn construct_input_from_contract((_idx, contract): (usize, &String)) -> fuel_tx::Input {
    fuel_tx::Input::Contract {
        utxo_id: fuel_tx::UtxoId::new(fuel_tx::Bytes32::zeroed(), 0),
        balance_root: fuel_tx::Bytes32::zeroed(),
        state_root: fuel_tx::Bytes32::zeroed(),
        contract_id: fuel_tx::ContractId::from_str(contract).unwrap(),
    }
}

fn construct_output_from_contract((idx, _contract): (usize, &String)) -> fuel_tx::Output {
    fuel_tx::Output::Contract {
        input_index: idx as u8, // probably safe unless a user inputs > u8::MAX inputs
        balance_root: fuel_tx::Bytes32::zeroed(),
        state_root: fuel_tx::Bytes32::zeroed(),
    }
}

/// Given some contracts, constructs the most basic input and output set that satisfies validation.
fn get_tx_inputs_and_outputs(
    contracts: Vec<String>,
) -> (Vec<fuel_tx::Input>, Vec<fuel_tx::Output>) {
    let inputs = contracts
        .iter()
        .enumerate()
        .map(construct_input_from_contract)
        .collect::<Vec<_>>();
    let outputs = contracts
        .iter()
        .enumerate()
        .map(construct_output_from_contract)
        .collect::<Vec<_>>();
    (inputs, outputs)
}
