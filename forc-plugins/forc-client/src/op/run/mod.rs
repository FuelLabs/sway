mod encode;
use crate::{
    cmd,
    util::{
        gas::get_script_gas_used,
        node_url::get_node_url,
        pkg::built_pkgs,
        tx::{TransactionBuilderExt, WalletSelectionMode, TX_SUBMIT_TIMEOUT_MS},
    },
};
use anyhow::{anyhow, bail, Context, Result};
use forc_pkg::{self as pkg, fuel_core_not_running, PackageManifestFile};
use forc_tracing::println_warning;
use forc_util::tx_utils::format_log_receipts;
use fuel_core_client::client::FuelClient;
use fuel_tx::{ContractId, Transaction, TransactionBuilder};
use fuels_accounts::provider::Provider;
use pkg::{manifest::build_profile::ExperimentalFlags, BuiltPackage};
use std::time::Duration;
use std::{path::PathBuf, str::FromStr};
use sway_core::language::parsed::TreeType;
use sway_core::BuildTarget;
use tokio::time::timeout;
use tracing::info;

use self::encode::ScriptCallHandler;

pub struct RanScript {
    pub receipts: Vec<fuel_tx::Receipt>,
}

/// Builds and runs script(s). If given path corresponds to a workspace, all runnable members will
/// be built and deployed.
///
/// Upon success, returns the receipts of each script in the order they are executed.
///
/// When running a single script, only that script's receipts are returned.
pub async fn run(command: cmd::Run) -> Result<Vec<RanScript>> {
    let mut command = command;
    if command.unsigned {
        println_warning("--unsigned flag is deprecated, please prefer using --default-signer. Assuming `--default-signer` is passed. This means your transaction will be signed by an account that is funded by fuel-core by default for testing purposes.");
        command.default_signer = true;
    }
    let mut receipts = Vec::new();
    let curr_dir = if let Some(path) = &command.pkg.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };
    let build_opts = build_opts_from_cmd(&command);
    let built_pkgs_with_manifest = built_pkgs(&curr_dir, &build_opts)?;
    for built in built_pkgs_with_manifest {
        if built
            .descriptor
            .manifest_file
            .check_program_type(&[TreeType::Script])
            .is_ok()
        {
            let pkg_receipts = run_pkg(&command, &built.descriptor.manifest_file, &built).await?;
            receipts.push(pkg_receipts);
        }
    }

    Ok(receipts)
}

pub async fn run_pkg(
    command: &cmd::Run,
    manifest: &PackageManifestFile,
    compiled: &BuiltPackage,
) -> Result<RanScript> {
    let node_url = get_node_url(&command.node, &manifest.network)?;

    let script_data = match (&command.data, &command.args) {
        (None, Some(args)) => {
            let minify_json_abi = true;
            let package_json_abi = compiled
                .json_abi_string(minify_json_abi)?
                .ok_or_else(|| anyhow::anyhow!("Missing json abi string"))?;
            let main_arg_handler = ScriptCallHandler::from_json_abi_str(&package_json_abi)?;
            let args = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();
            main_arg_handler.encode_arguments(args.as_slice())?
        }
        (Some(_), Some(_)) => {
            bail!("Both --args and --data provided, must choose one.")
        }
        _ => {
            let input_data = command.data.as_deref().unwrap_or("");
            let data = input_data.strip_prefix("0x").unwrap_or(input_data);
            hex::decode(data).expect("Invalid hex")
        }
    };

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
    let wallet_mode = if command.manual_signing {
        WalletSelectionMode::Manual
    } else {
        WalletSelectionMode::ForcWallet
    };

    let mut tb = TransactionBuilder::script(compiled.bytecode.bytes.clone(), script_data);
    tb.maturity(command.maturity.maturity.into())
        .add_contracts(contract_ids);

    let provider = Provider::connect(node_url.clone()).await?;

    let script_gas_limit = if compiled.bytecode.bytes.is_empty() {
        0
    } else if let Some(script_gas_limit) = command.gas.script_gas_limit {
        script_gas_limit
    // Dry run tx and get `gas_used`
    } else {
        get_script_gas_used(tb.clone().finalize_without_signature_inner(), &provider).await?
    };
    tb.script_gas_limit(script_gas_limit);

    let tx = tb
        .finalize_signed(
            Provider::connect(node_url.clone()).await?,
            command.default_signer,
            command.signing_key,
            wallet_mode,
        )
        .await?;

    if command.dry_run {
        info!("{:?}", tx);
        Ok(RanScript { receipts: vec![] })
    } else {
        let receipts = try_send_tx(
            node_url.as_str(),
            &tx.into(),
            command.pretty_print,
            command.simulate,
        )
        .await?;
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
        .with_context(|| format!("timeout waiting for {:?} to be included in a block", tx))?,
        Err(_) => Err(fuel_core_not_running(node_url)),
    }
}

async fn send_tx(
    client: &FuelClient,
    tx: &Transaction,
    pretty_print: bool,
    simulate: bool,
) -> Result<Vec<fuel_tx::Receipt>> {
    let outputs = {
        if !simulate {
            let status = client.submit_and_await_commit(tx).await?;

            match status {
                fuel_core_client::client::types::TransactionStatus::Success {
                    receipts, ..
                } => receipts,
                fuel_core_client::client::types::TransactionStatus::Failure {
                    receipts, ..
                } => receipts,
                _ => vec![],
            }
        } else {
            let txs = vec![tx.clone()];
            let receipts = client.dry_run(txs.as_slice()).await?;
            let receipts = receipts
                .first()
                .map(|tx| &tx.result)
                .map(|res| res.receipts());
            match receipts {
                Some(receipts) => receipts.to_vec(),
                None => vec![],
            }
        }
    };
    if !outputs.is_empty() {
        info!("{}", format_log_receipts(&outputs, pretty_print)?);
    }
    Ok(outputs)
}

fn build_opts_from_cmd(cmd: &cmd::Run) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.pkg.path.clone(),
            offline: cmd.pkg.offline,
            terse: cmd.pkg.terse,
            locked: cmd.pkg.locked,
            output_directory: cmd.pkg.output_directory.clone(),
            json_abi_with_callpaths: cmd.pkg.json_abi_with_callpaths,
            ipfs_node: cmd.pkg.ipfs_node.clone().unwrap_or_default(),
        },
        print: pkg::PrintOpts {
            ast: cmd.print.ast,
            dca_graph: cmd.print.dca_graph.clone(),
            dca_graph_url_format: cmd.print.dca_graph_url_format.clone(),
            asm: cmd.print.asm(),
            bytecode: cmd.print.bytecode,
            ir: cmd.print.ir(),
            reverse_order: cmd.print.reverse_order,
        },
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify.json_abi,
            json_storage_slots: cmd.minify.json_storage_slots,
        },
        build_target: BuildTarget::default(),
        build_profile: cmd.build_profile.build_profile.clone(),
        release: cmd.build_profile.release,
        error_on_warnings: cmd.build_profile.error_on_warnings,
        time_phases: cmd.print.time_phases,
        metrics_outfile: cmd.print.metrics_outfile.clone(),
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file.clone(),
        tests: false,
        member_filter: pkg::MemberFilter::only_scripts(),
        experimental: ExperimentalFlags {
            new_encoding: !cmd.no_encoding_v1,
        },
    }
}
