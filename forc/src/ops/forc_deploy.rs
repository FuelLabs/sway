use core_lang::parse;
use fuel_tx::{crypto::hash, ContractId, Output, Salt, Transaction};
use tx_client::client::TxClient;

use crate::cli::{BuildCommand, DeployCommand};
use crate::ops::forc_build;
use crate::utils::cli_error::CliError;
use crate::utils::{constants, helpers};
use constants::{DEFAULT_NODE_URL, SWAY_CONTRACT, SWAY_LIBRARY, SWAY_PREDICATE, SWAY_SCRIPT};
use helpers::{find_manifest_dir, get_main_file, read_manifest};

pub async fn deploy(_: DeployCommand) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir()?;

    match find_manifest_dir(&curr_dir) {
        Some(manifest_dir) => {
            let manifest = read_manifest(&manifest_dir)?;
            let project_name = &manifest.project.name;
            let main_file = get_main_file(&manifest, &manifest_dir)?;

            // parse the main file and check is it a contract
            match parse(main_file) {
                core_lang::CompileResult::Ok {
                    value: parse_tree,
                    warnings: _,
                    errors: _,
                } => {
                    if let Some(_) = &parse_tree.contract_ast {
                        let build_command = BuildCommand {
                            path: None,
                            print_asm: false,
                            binary_outfile: None,
                            offline_mode: false,
                        };

                        let compiled_contract = forc_build::build(build_command)?;
                        let tx = create_contract_tx(compiled_contract);

                        let node_url = match &manifest.network {
                            Some(network) => &network.url,
                            _ => DEFAULT_NODE_URL,
                        };

                        let client = TxClient::new(node_url)?;

                        match client.transact(&tx).await {
                            Ok(logs) => {
                                println!("{:?}", logs);
                                Ok(())
                            }
                            Err(e) => Err(e.to_string().into()),
                        }
                    } else {
                        let parse_type = {
                            if parse_tree.script_ast.is_some() {
                                SWAY_SCRIPT
                            } else if parse_tree.predicate_ast.is_some() {
                                SWAY_PREDICATE
                            } else {
                                SWAY_LIBRARY
                            }
                        };

                        Err(CliError::wrong_sway_type(
                            project_name,
                            SWAY_CONTRACT,
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
        None => Err(CliError::manifest_file_missing(curr_dir)),
    }
}

fn create_contract_tx(compiled_contract: Vec<u8>) -> Transaction {
    let gas_price = 0;
    let gas_limit = 10000000;
    let maturity = 0;
    let bytecode_witness_index = 0;
    let witnesses = vec![compiled_contract.into()];

    let salt = Salt::new([0; 32]);
    let static_contracts = vec![];
    let inputs = vec![];

    let zero_hash = hash("0".as_bytes());

    let outputs = vec![Output::ContractCreated {
        contract_id: ContractId::new(zero_hash.into()),
    }];

    Transaction::create(
        gas_price,
        gas_limit,
        maturity,
        bytecode_witness_index,
        salt,
        static_contracts,
        inputs,
        outputs,
        witnesses,
    )
}
