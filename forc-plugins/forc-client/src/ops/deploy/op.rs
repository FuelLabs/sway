use anyhow::{bail, Result};
use forc_pkg::{BuildOptions, PackageManifestFile};
use fuel_gql_client::client::FuelClient;
use fuel_tx::{Output, Salt, TransactionBuilder};
use fuel_vm::prelude::*;
use std::path::PathBuf;
use sway_core::language::parsed::TreeType;
use sway_utils::constants::DEFAULT_NODE_URL;
use tracing::info;

use crate::ops::tx_util::{TransactionBuilderExt, TxParameters};

use super::cmd::DeployCommand;

pub async fn deploy(command: DeployCommand) -> Result<fuel_tx::ContractId> {
    let curr_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = PackageManifestFile::from_dir(&curr_dir)?;
    manifest.check_program_type(vec![TreeType::Contract])?;

    let node_url = match &manifest.network {
        Some(network) => &network.url,
        _ => DEFAULT_NODE_URL,
    };
    let node_url = command.url.unwrap_or_else(|| node_url.to_string());
    let client = FuelClient::new(node_url)?;

    let build_options = BuildOptions {
        path: command.path,
        print_ast: command.print_ast,
        print_finalized_asm: command.print_finalized_asm,
        print_intermediate_asm: command.print_intermediate_asm,
        print_ir: command.print_ir,
        binary_outfile: command.binary_outfile,
        offline_mode: command.offline_mode,
        debug_outfile: command.debug_outfile,
        terse_mode: command.terse_mode,
        output_directory: command.output_directory,
        minify_json_abi: command.minify_json_abi,
        minify_json_storage_slots: command.minify_json_storage_slots,
        locked: command.locked,
        build_profile: command.build_profile,
        release: command.release,
        time_phases: command.time_phases,
        tests: false,
    };
    let compiled = forc_pkg::build_with_options(build_options)?;

    let bytecode = compiled.bytecode.clone().into();
    let salt = Salt::new([0; 32]);
    let mut storage_slots = compiled.storage_slots;
    storage_slots.sort();
    let contract = Contract::from(compiled.bytecode.clone());
    let root = contract.root();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let contract_id = contract.id(&salt, &root, &state_root);
    info!("Contract id: 0x{}", hex::encode(contract_id));
    let tx = TransactionBuilder::create(bytecode, salt, storage_slots.clone())
        .params(TxParameters::new(command.gas_limit, command.gas_price))
        .add_output(Output::contract_created(contract_id, state_root))
        .finalize_signed(client.clone(), command.unsigned, command.signing_key)
        .await?;

    match client.submit(&tx).await {
        Ok(logs) => {
            info!("Logs:\n{:?}", logs);
            Ok(contract_id)
        }
        Err(e) => bail!("{e}"),
    }
}
