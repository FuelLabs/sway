use crate::{
    cmd,
    util::{
        pkg::built_pkgs_with_manifest,
        tx::{TransactionBuilderExt, TX_SUBMIT_TIMEOUT_MS},
    },
};
use anyhow::{bail, Context, Result};
use forc_pkg::{self as pkg, PackageManifestFile};
use fuel_core_client::client::types::TransactionStatus;
use fuel_core_client::client::FuelClient;
use fuel_tx::{Output, TransactionBuilder};
use fuel_vm::prelude::*;
use futures::FutureExt;
use pkg::BuiltPackage;
use std::path::PathBuf;
use std::time::Duration;
use sway_core::language::parsed::TreeType;
use sway_core::BuildTarget;
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

    // If a salt was specified but we have more than one member to build, there
    // may be ambiguity in how the salt should be applied, especially if the
    // workspace contains multiple contracts, and especially if one contract
    // member is the dependency of another (in which case salt should be
    // specified under `[contract- dependencies]`). Considering this, we have a
    // simple check to ensure that we only accept salt when deploying a single
    // package. In the future, we can consider relaxing this to allow for
    // specifying a salt for workspace deployment, as long as there is only one
    // root contract member in the package graph.
    if command.salt.salt.is_some() && built_pkgs_with_manifest.len() > 1 {
        bail!(
            "A salt was specified when attempting to deploy a workspace with more than one member.
              If you wish to deploy a contract member with salt, deploy the member individually.
              If you wish to specify the salt for a contract dependency, \
            please do so within the `[contract-dependencies]` table."
        )
    }

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
    let node_url = command
        .node_url
        .as_deref()
        .or_else(|| manifest.network.as_ref().map(|nw| &nw.url[..]))
        .unwrap_or(crate::default::NODE_URL);
    let client = FuelClient::new(node_url)?;

    let bytecode = &compiled.bytecode.bytes;
    let salt = match (command.salt.salt, command.random_salt) {
        (Some(salt), false) => salt,
        (None, true) => rand::random(),
        (None, false) => Default::default(),
        (Some(_), true) => {
            bail!("Both `--salt` and `--random-salt` were specified: must choose one")
        }
    };
    let mut storage_slots = compiled.storage_slots.clone();
    storage_slots.sort();

    let contract = Contract::from(bytecode.clone());
    let root = contract.root();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let contract_id = contract.id(&salt, &root, &state_root);
    info!("Contract id: 0x{}", hex::encode(contract_id));

    let tx = TransactionBuilder::create(bytecode.as_slice().into(), salt, storage_slots.clone())
        .gas_limit(command.gas.limit)
        .gas_price(command.gas.price)
        // TODO: Spec says maturity should be u32, but fuel-tx wants u64.
        .maturity(u64::from(command.maturity.maturity))
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
            json_abi_with_callpaths: cmd.pkg.json_abi_with_callpaths,
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
        error_on_warnings: cmd.build_profile.error_on_warnings,
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file.clone(),
        build_target: BuildTarget::default(),
        tests: false,
        const_inject_map,
        member_filter: pkg::MemberFilter::only_contracts(),
    }
}
