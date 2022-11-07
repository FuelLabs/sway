use anyhow::{anyhow, bail, Context, Result};
use forc_pkg::{self as pkg, fuel_core_not_running, PackageManifestFile};
use fuel_gql_client::client::FuelClient;
use fuel_tx::{ContractId, Transaction, TransactionBuilder, UniqueIdentifier};
use futures::TryFutureExt;
use pkg::manifest::ManifestFile;
use pkg::{BuildPlan, BuiltPackage};
use std::time::Duration;
use std::{path::PathBuf, str::FromStr};
use sway_core::language::parsed::TreeType;
use tokio::time::timeout;
use tracing::info;

use crate::ops::tx_util::{TransactionBuilderExt, TxParameters, TX_SUBMIT_TIMEOUT_MS};

use super::cmd::RunCommand;

pub const NODE_URL: &str = "http://127.0.0.1:4000";

/// Builds and runs script(s). If given path corresponds to a workspace, all runnable members
/// of the workspace is going to be built and deployed.
///
/// Returns the list of receipts for each runnable member, if a single pkg is ran, list contains
/// just one list of receipts. Otherwise, it contains a list of receipts for each member of the workspace in
/// the order they are ran.
pub async fn run(command: RunCommand) -> Result<Vec<Vec<fuel_tx::Receipt>>> {
    let mut receipts = Vec::new();
    let curr_dir = if let Some(path) = &command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
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
            .check_program_type(vec![TreeType::Script])
            .is_ok()
        {
            if let Some(built_pkg) = built_packages.get(&member_manifest.project.name) {
                let pkg_receipts = run_pkg(&command, member_manifest, built_pkg).await?;
                receipts.push(pkg_receipts);
            }
        }
    }

    Ok(receipts)
}

pub async fn run_pkg(
    command: &RunCommand,
    manifest: &PackageManifestFile,
    compiled: &BuiltPackage,
) -> Result<Vec<fuel_tx::Receipt>> {
    let input_data = command.data.as_deref().unwrap_or("");
    let data = format_hex_data(input_data);
    let script_data = hex::decode(data).expect("Invalid hex");

    let node_url = command
        .node_url
        .as_deref()
        .or_else(|| manifest.network.as_ref().map(|nw| &nw.url[..]))
        .unwrap_or(NODE_URL);
    let client = FuelClient::new(node_url)?;
    let contract_ids: Vec<ContractId> = command
        .contract
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|contract_id| ContractId::from_str(contract_id).unwrap())
        .collect();
    let tx = TransactionBuilder::script(compiled.bytecode.clone(), script_data)
        .params(TxParameters::new(command.gas_limit, command.gas_price))
        .add_contracts(contract_ids)
        .finalize_signed(client.clone(), command.unsigned, command.signing_key)
        .await?;
    if command.dry_run {
        info!("{:?}", tx);
        Ok(vec![])
    } else {
        try_send_tx(node_url, &tx.into(), command.pretty_print, command.simulate).await
    }
}

async fn try_send_tx(
    node_url: &str,
    tx: &Transaction,
    pretty_print: bool,
    simulate: bool,
) -> Result<Vec<fuel_tx::Receipt>> {
    let client = FuelClient::new(node_url)?;

    match client.health().await {
        Ok(_) => timeout(
            Duration::from_millis(TX_SUBMIT_TIMEOUT_MS),
            send_tx(&client, tx, pretty_print, simulate),
        )
        .await
        .with_context(|| format!("timeout waiting for {} to be included in a block", tx.id()))?,
        Err(_) => Err(fuel_core_not_running(node_url)),
    }
}

async fn send_tx(
    client: &FuelClient,
    tx: &Transaction,
    pretty_print: bool,
    simulate: bool,
) -> Result<Vec<fuel_tx::Receipt>> {
    let id = format!("{:#x}", tx.id());
    let outputs = {
        if !simulate {
            client
                .submit_and_await_commit(tx)
                .and_then(|_| client.receipts(id.as_str()))
                .await
        } else {
            client
                .dry_run(tx)
                .and_then(|_| client.receipts(id.as_str()))
                .await
        }
    };

    match outputs {
        Ok(logs) => {
            print_receipt_output(&logs, pretty_print)?;
            Ok(logs)
        }
        Err(e) => bail!("{e}"),
    }
}

// cut '0x' from the start
fn format_hex_data(data: &str) -> &str {
    data.strip_prefix("0x").unwrap_or(data)
}

fn print_receipt_output(receipts: &Vec<fuel_tx::Receipt>, pretty_print: bool) -> Result<()> {
    let mut receipt_to_json_array = serde_json::to_value(receipts)?;
    for (rec_index, receipt) in receipts.iter().enumerate() {
        let rec_value = receipt_to_json_array.get_mut(rec_index).ok_or_else(|| {
            anyhow!(
                "Serialized receipts does not contain {} th index",
                rec_index
            )
        })?;
        match receipt {
            fuel_tx::Receipt::LogData { data, .. } => {
                if let Some(v) = rec_value.pointer_mut("/LogData/data") {
                    *v = hex::encode(data).into();
                }
            }
            fuel_tx::Receipt::ReturnData { data, .. } => {
                if let Some(v) = rec_value.pointer_mut("/ReturnData/data") {
                    *v = hex::encode(data).into();
                }
            }
            _ => {}
        }
    }
    if pretty_print {
        info!("{}", serde_json::to_string_pretty(&receipt_to_json_array)?);
    } else {
        info!("{}", serde_json::to_string(&receipt_to_json_array)?);
    }
    Ok(())
}

fn build_opts_from_cmd(cmd: &RunCommand) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.path.clone(),
            offline: false,
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
