use anyhow::{anyhow, bail, Context, Result};
use forc_pkg::{self as pkg, PackageManifestFile};
use fuel_gql_client::client::types::TransactionStatus;
use fuel_gql_client::{
    client::FuelClient,
    fuel_tx::{Output, Salt, TransactionBuilder},
    fuel_vm::prelude::*,
};
use futures::FutureExt;
use pkg::manifest::ManifestFile;
use pkg::{BuildPlan, BuiltPackage};
use std::path::PathBuf;
use std::time::Duration;
use sway_core::language::parsed::TreeType;
use sway_utils::constants::DEFAULT_NODE_URL;
use tracing::info;

use crate::ops::tx_util::{TransactionBuilderExt, TxParameters, TX_SUBMIT_TIMEOUT_MS};

use super::cmd::DeployCommand;

/// Builds and deploys contract(s). If given path corresponds to a workspace, all deployable members
/// of the workspace is going to be built and deployed.
///
/// Returns the list of contract id(s), if the deployment is done for a single pkg, list contains
/// just one contract id. Otherwise, it contains a contract id for each member of the workspace in
/// the order of deployment.
pub async fn deploy(command: DeployCommand) -> Result<Vec<fuel_tx::ContractId>> {
    let mut contract_ids = Vec::new();
    let curr_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest_file = ManifestFile::from_dir(&curr_dir)?;
    let member_manifests = manifest_file.member_manifests()?;
    let lock_path = manifest_file.lock_path()?;
    let build_opts = build_opts_from_cmd(&command);
    let build_plan = BuildPlan::from_lock_and_manifests(
        &lock_path,
        &manifest_file.member_manifests()?,
        build_opts.pkg.locked,
        build_opts.pkg.offline,
    )?;
    let built = pkg::build_with_options(build_opts)?;
    let built_packages = match built {
        pkg::Built::Package(built_pkg) => {
            let pkg_name = member_manifests
                .keys()
                .next()
                .ok_or_else(|| anyhow!("built package is missing"))?;
            std::iter::once((pkg_name.clone(), *built_pkg)).collect()
        }
        pkg::Built::Workspace(built_workspace) => built_workspace,
    };
    let graph = build_plan.graph();
    for member_node_ix in build_plan.member_nodes() {
        let pkg_name = &graph[member_node_ix].name;
        let member_manifest = member_manifests
            .get(pkg_name)
            .ok_or_else(|| anyhow!("Member manifest file is missing"))?;
        if member_manifest
            .check_program_type(vec![TreeType::Contract])
            .is_ok()
        {
            if let Some(built_pkg) = built_packages.get(&member_manifest.project.name) {
                let contract_id = deploy_pkg(&command, member_manifest, built_pkg).await;
                contract_ids.push(contract_id?);
            }
        }
    }
    Ok(contract_ids)
}

/// Deploy a single pkg given deploy command and the manifest file
pub async fn deploy_pkg(
    command: &DeployCommand,
    manifest: &PackageManifestFile,
    compiled: &BuiltPackage,
) -> Result<fuel_tx::ContractId> {
    let node_url = match &manifest.network {
        Some(network) => &network.url,
        _ => DEFAULT_NODE_URL,
    };

    let node_url = command.url.as_deref().unwrap_or(node_url);
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
        .params(TxParameters::new(command.gas_limit, command.gas_price))
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
            TransactionStatus::Failure { reason, .. } => {
                bail!(
                    "contract {} failed to deploy due to an error: {}",
                    &contract_id,
                    reason
                )
            }
        },
        Err(e) => bail!("{e}"),
    });

    // submit contract deployment with a timeout
    tokio::time::timeout(
        Duration::from_millis(TX_SUBMIT_TIMEOUT_MS),
        deployment_request,
    )
    .await
    .with_context(|| {
        format!(
            "Timed out waiting for contract {} to deploy. The transaction may have been dropped.",
            &contract_id
        )
    })?
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
