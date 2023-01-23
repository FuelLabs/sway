use anyhow::{anyhow, bail, Context, Result};
use forc_pkg::{self as pkg, fuel_core_not_running, PackageManifestFile};
use forc_util::format_log_receipts;
use fuel_gql_client::client::FuelClient;
use fuel_tx::{ContractId, Transaction, TransactionBuilder, UniqueIdentifier};
use futures::TryFutureExt;
use pkg::BuiltPackage;
use std::time::Duration;
use std::{path::PathBuf, str::FromStr};
use sway_core::language::parsed::TreeType;
use sway_core::BuildTarget;
use tokio::time::timeout;
use tracing::info;

use crate::ops::pkg_util::built_pkgs_with_manifest;
use crate::ops::tx_util::{TransactionBuilderExt, TxParameters, TX_SUBMIT_TIMEOUT_MS};

use super::cmd::RunCommand;

pub const NODE_URL: &str = "http://127.0.0.1:4000";

pub struct RanScript {
    pub receipts: Vec<fuel_tx::Receipt>,
}

/// Builds and runs script(s). If given path corresponds to a workspace, all runnable members will
/// be built and deployed.
///
/// Upon success, returns the receipts of each script in the order they are executed.
///
/// When running a single script, only that script's receipts are returned.
pub async fn run(command: RunCommand) -> Result<Vec<RanScript>> {
    let mut receipts = Vec::new();
    let curr_dir = if let Some(path) = &command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };
    let build_opts = build_opts_from_cmd(&command);
    let built_pkgs_with_manifest = built_pkgs_with_manifest(&curr_dir, build_opts)?;
    for (member_manifest, built_pkg) in built_pkgs_with_manifest {
        if member_manifest
            .check_program_type(vec![TreeType::Script])
            .is_ok()
        {
            let pkg_receipts = run_pkg(&command, &member_manifest, &built_pkg).await?;
            receipts.push(pkg_receipts);
        }
    }

    Ok(receipts)
}

pub async fn run_pkg(
    command: &RunCommand,
    manifest: &PackageManifestFile,
    compiled: &BuiltPackage,
) -> Result<RanScript> {
    let input_data = command.data.as_deref().unwrap_or("");
    let data = input_data.strip_prefix("0x").unwrap_or(input_data);
    let script_data = hex::decode(data).expect("Invalid hex");

    let node_url = command
        .node_url
        .as_deref()
        .or_else(|| manifest.network.as_ref().map(|nw| &nw.url[..]))
        .unwrap_or(NODE_URL);
    let client = FuelClient::new(node_url)?;
    let contract_ids = command
        .contract
        .as_ref()
        .into_iter()
        .flat_map(|contracts| contracts.iter())
        .map(|contract| {
            ContractId::from_str(contract)
                .map_err(|e| anyhow!("Failed to parse contract id: {}", e))
        })
        .collect::<Result<Vec<ContractId>>>()?;
    let tx = TransactionBuilder::script(compiled.bytecode.clone(), script_data)
        .params(TxParameters::new(command.gas_limit, command.gas_price))
        .add_contracts(contract_ids)
        .finalize_signed(client.clone(), command.unsigned, command.signing_key)
        .await?;
    if command.dry_run {
        info!("{:?}", tx);
        Ok(RanScript { receipts: vec![] })
    } else {
        let receipts =
            try_send_tx(node_url, &tx.into(), command.pretty_print, command.simulate).await?;
        Ok(RanScript { receipts })
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
            info!("{}", format_log_receipts(&logs, pretty_print)?);
            Ok(logs)
        }
        Err(e) => bail!("{e}"),
    }
}

fn build_opts_from_cmd(cmd: &RunCommand) -> pkg::BuildOpts {
    let inject_map = std::collections::HashMap::new();
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
            dca_graph: cmd.print_dca_graph,
            finalized_asm: cmd.print_finalized_asm,
            intermediate_asm: cmd.print_intermediate_asm,
            ir: cmd.print_ir,
        },
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify_json_abi,
            json_storage_slots: cmd.minify_json_storage_slots,
        },
        build_target: BuildTarget::default(),
        build_profile: cmd.build_profile.clone(),
        release: cmd.release,
        time_phases: cmd.time_phases,
        binary_outfile: cmd.binary_outfile.clone(),
        debug_outfile: cmd.debug_outfile.clone(),
        tests: false,
        inject_map,
    }
}
