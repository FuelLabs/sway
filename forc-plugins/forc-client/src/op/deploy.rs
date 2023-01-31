use crate::{
    cmd,
    util::{
        pkg::built_pkgs_with_manifest,
        tx::{TransactionBuilderExt, TX_SUBMIT_TIMEOUT_MS},
    },
};
use anyhow::{bail, Context, Result};
use forc_pkg::{self as pkg, PackageManifestFile};
use fuel_gql_client::client::types::TransactionStatus;
use fuel_gql_client::{
    client::FuelClient,
    fuel_tx::{Output, Salt, TransactionBuilder},
    fuel_vm::prelude::*,
};
use futures::FutureExt;
use pkg::BuiltPackage;
use std::path::PathBuf;
use std::time::Duration;
use sway_core::language::parsed::TreeType;
use sway_core::BuildTarget;
use sway_utils::constants::DEFAULT_NODE_URL;
use tracing::info;

pub struct DeployedContract {
    pub id: fuel_tx::ContractId,
}

/// Builds and deploys contract(s). If the given path corresponds to a workspace, all deployable members
/// will be built and deployed.
///
/// Upon success, returns the ID of each deployed contract in order of deployment.
///
/// When deploying a single contract, only that contract's ID is returned.
pub async fn deploy(command: cmd::Deploy) -> Result<Vec<DeployedContract>> {
    let mut contract_ids = Vec::new();
    let curr_dir = if let Some(ref path) = command.pkg.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let build_opts = build_opts_from_cmd(&command);
    let built_pkgs_with_manifest = built_pkgs_with_manifest(&curr_dir, build_opts)?;
    for (member_manifest, built_pkg) in built_pkgs_with_manifest {
        if member_manifest
            .check_program_type(vec![TreeType::Contract])
            .is_ok()
        {
            let contract_id = deploy_pkg(&command, &member_manifest, &built_pkg).await?;
            contract_ids.push(contract_id);
        }
    }
    Ok(contract_ids)
}

/// Deploy a single pkg given deploy command and the manifest file
pub async fn deploy_pkg(
    command: &cmd::Deploy,
    manifest: &PackageManifestFile,
    compiled: &BuiltPackage,
) -> Result<DeployedContract> {
    let node_url = match &manifest.network {
        Some(network) => &network.url,
        _ => DEFAULT_NODE_URL,
    };

    let node_url = command.node_url.as_deref().unwrap_or(node_url);
    let client = FuelClient::new(node_url)?;

    let bytecode = compiled.bytecode.clone().into();
    let salt = Salt::new([0; 32]);
    let mut storage_slots = compiled.storage_slots.clone();
    storage_slots.sort();

    let contract = Contract::from(compiled.bytecode.clone());
    let root = contract.root();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let contract_id = contract.id(&salt, &root, &state_root);
    info!("Contract id: 0x{}", hex::encode(contract_id));

    let tx = TransactionBuilder::create(bytecode, salt, storage_slots.clone())
        .gas_limit(command.gas.limit)
        .gas_price(command.gas.price)
        .add_output(Output::contract_created(contract_id, state_root))
        .finalize_signed(client.clone(), command.unsigned, command.signing_key)
        .await?;

    let tx = Transaction::from(tx);

    let deployment_request = client.submit_and_await_commit(&tx).map(|res| match res {
        Ok(logs) => match logs {
            TransactionStatus::Submitted { .. } => {
                bail!("contract {} deployment timed out", &contract_id);
            }
            TransactionStatus::Success { block_id, .. } => {
                info!("contract {} deployed in block {}", &contract_id, &block_id);
                Ok(contract_id)
            }
            e => {
                bail!(
                    "contract {} failed to deploy due to an error: {:?}",
                    &contract_id,
                    e
                )
            }
        },
        Err(e) => bail!("{e}"),
    });

    // submit contract deployment with a timeout
    let contract_id = tokio::time::timeout(
        Duration::from_millis(TX_SUBMIT_TIMEOUT_MS),
        deployment_request,
    )
    .await
    .with_context(|| {
        format!(
            "Timed out waiting for contract {} to deploy. The transaction may have been dropped.",
            &contract_id
        )
    })??;
    Ok(DeployedContract { id: contract_id })
}

fn build_opts_from_cmd(cmd: &cmd::Deploy) -> pkg::BuildOpts {
    let const_inject_map = std::collections::HashMap::new();
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.pkg.path.clone(),
            offline: cmd.pkg.offline,
            terse: cmd.pkg.terse,
            locked: cmd.pkg.locked,
            output_directory: cmd.pkg.output_directory.clone(),
        },
        print: pkg::PrintOpts {
            ast: cmd.print.ast,
            dca_graph: cmd.print.dca_graph,
            finalized_asm: cmd.print.finalized_asm,
            intermediate_asm: cmd.print.intermediate_asm,
            ir: cmd.print.ir,
        },
        time_phases: cmd.print.time_phases,
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify.json_abi,
            json_storage_slots: cmd.minify.json_storage_slots,
        },
        build_profile: cmd.build_profile.build_profile.clone(),
        release: cmd.build_profile.release,
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file.clone(),
        build_target: BuildTarget::default(),
        tests: false,
        const_inject_map,
    }
}
