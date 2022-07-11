use crate::ops::forc_build;
use crate::{
    cli::{BuildCommand, DeployCommand},
    utils::SWAY_GIT_TAG,
};
use anyhow::{bail, Result};
use forc_pkg::ManifestFile;
use fuel_gql_client::client::FuelClient;
use fuel_tx::{Output, Salt, StorageSlot, Transaction};
use fuel_vm::prelude::*;
use std::path::PathBuf;
use sway_core::TreeType;
use sway_utils::constants::DEFAULT_NODE_URL;
use tracing::info;

pub async fn deploy(command: DeployCommand) -> Result<fuel_tx::ContractId> {
    let curr_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&curr_dir, SWAY_GIT_TAG)?;
    manifest.check_program_type(vec![TreeType::Contract])?;

    let DeployCommand {
        path,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        binary_outfile,
        debug_outfile,
        offline_mode,
        silent_mode,
        output_directory,
        minify_json_abi,
        minify_json_storage_slots,
        locked,
        url,
        build_profile,
        release,
        time_phases,
    } = command;

    let build_command = BuildCommand {
        path,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        binary_outfile,
        offline_mode,
        debug_outfile,
        silent_mode,
        output_directory,
        minify_json_abi,
        minify_json_storage_slots,
        locked,
        build_profile,
        release,
        time_phases,
    };

    let compiled = forc_build::build(build_command)?;
    let (tx, contract_id) = create_contract_tx(
        compiled.bytecode,
        Vec::<fuel_tx::Input>::new(),
        Vec::<fuel_tx::Output>::new(),
        compiled.storage_slots,
    );

    let node_url = match &manifest.network {
        Some(network) => &network.url,
        _ => DEFAULT_NODE_URL,
    };

    let node_url = url.unwrap_or_else(|| node_url.to_string());

    let client = FuelClient::new(node_url)?;

    match client.submit(&tx).await {
        Ok(logs) => {
            info!("Logs:\n{:?}", logs);
            Ok(contract_id)
        }
        Err(e) => bail!("{e}"),
    }
}

fn create_contract_tx(
    compiled_contract: Vec<u8>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    storage_slots: Vec<StorageSlot>,
) -> (Transaction, fuel_tx::ContractId) {
    let gas_price = 0;
    let gas_limit = fuel_tx::ConsensusParameters::default().max_gas_per_tx;
    let byte_price = 0;
    let maturity = 0;
    let bytecode_witness_index = 0;
    let witnesses = vec![compiled_contract.clone().into()];

    let salt = Salt::new([0; 32]);
    let static_contracts = vec![];

    let contract = Contract::from(compiled_contract);
    let root = contract.root();

    // The VM currently requires that storage slots are sorted but this shouldn't be neessary.
    // Downstream tooling should do the sorting themselves.
    // Ref: https://github.com/FuelLabs/fuel-tx/issues/153
    let mut storage_slots = storage_slots;
    storage_slots.sort();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let id = contract.id(&salt, &root, &state_root);
    info!("Contract id: 0x{}", hex::encode(id));
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
