use anyhow::{bail, Result};
use forc_pkg::{self as pkg, PackageManifestFile};
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
    let node_url = command.url.as_deref().unwrap_or(node_url);
    let client = FuelClient::new(node_url)?;

    let build_opts = build_opts_from_cmd(&command);
    let compiled = forc_pkg::build_package_with_options(&manifest, build_opts)?;

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

fn build_opts_from_cmd(cmd: &DeployCommand) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.path.clone(),
            offline: cmd.offline_mode,
            terse: cmd.terse_mode,
            locked: cmd.locked,
            output_directory: cmd.output_directory.clone(),
        },
        print: pkg::PrintOpts {
            ast: cmd.print_ast,
            finalized_asm: cmd.print_finalized_asm,
            intermediate_asm: cmd.print_intermediate_asm,
            ir: cmd.print_ir,
        },
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify_json_abi,
            json_storage_slots: cmd.minify_json_storage_slots,
        },
        build_profile: cmd.build_profile.clone(),
        release: cmd.release,
        time_phases: cmd.time_phases,
        binary_outfile: cmd.binary_outfile.clone(),
        debug_outfile: cmd.debug_outfile.clone(),
        tests: false,
    }
}
