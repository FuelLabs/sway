use crate::cli::{BuildCommand, DeployCommand};
use crate::ops::forc_build;
use crate::utils::cli_error::check_tree_type;
use anyhow::{anyhow, Result};
use fuel_gql_client::client::FuelClient;
use fuel_tx::{Output, Salt, Transaction};
use fuel_vm::prelude::*;
use std::path::PathBuf;
use sway_utils::constants::{DEFAULT_NODE_URL, SWAY_CONTRACT};

pub async fn deploy(command: DeployCommand) -> Result<fuel_tx::ContractId> {
    let curr_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = check_tree_type(curr_dir, SWAY_CONTRACT)?;

    let DeployCommand {
        path,
        use_ir,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        binary_outfile,
        debug_outfile,
        offline_mode,
        silent_mode,
        output_directory,
        minify_json_abi,
    } = command;

    let build_command = BuildCommand {
        path,
        use_ir,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        binary_outfile,
        offline_mode,
        debug_outfile,
        silent_mode,
        output_directory,
        minify_json_abi,
    };

    let compiled = forc_build::build(build_command)?;
    let (tx, contract_id) = create_contract_tx(
        compiled.bytecode,
        Vec::<fuel_tx::Input>::new(),
        Vec::<fuel_tx::Output>::new(),
    );

    let node_url = match &manifest.network {
        Some(network) => &network.url,
        _ => DEFAULT_NODE_URL,
    };

    let client = FuelClient::new(node_url)?;

    match client.submit(&tx).await {
        Ok(logs) => {
            println!("Logs:\n{:?}", logs);
            Ok(contract_id)
        }
        Err(e) => Err(anyhow!("{}", e)),
    }
}

fn create_contract_tx(
    compiled_contract: Vec<u8>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
) -> (Transaction, fuel_tx::ContractId) {
    let gas_price = 0;
    let gas_limit = fuel_tx::consts::MAX_GAS_PER_TX;
    let byte_price = 0;
    let maturity = 0;
    let bytecode_witness_index = 0;
    let witnesses = vec![compiled_contract.clone().into()];

    let salt = Salt::new([0; 32]);
    let static_contracts = vec![];
    let storage_slots = vec![];

    let contract = Contract::from(compiled_contract);
    let root = contract.root();
    let state_root = Contract::default_state_root();
    let id = contract.id(&salt, &root, &state_root);
    println!("Contract id: 0x{}", hex::encode(id));
    let outputs = [
        &[Output::ContractCreated {
            contract_id: id,
            state_root,
        }],
        &outputs[..],
    ]
    .concat();

    (
        Transaction::create(
            gas_price,
            gas_limit,
            byte_price,
            maturity,
            bytecode_witness_index,
            salt,
            static_contracts,
            storage_slots,
            inputs,
            outputs,
            witnesses,
        ),
        id,
    )
}
