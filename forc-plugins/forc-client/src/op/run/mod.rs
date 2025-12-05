mod encode;
use crate::{
    cmd,
    constants::TX_SUBMIT_TIMEOUT_MS,
    util::{
        pkg::built_pkgs,
        tx::{prompt_forc_wallet_password, select_account, SignerSelectionMode},
    },
};
use anyhow::{anyhow, bail, Context, Result};
use forc::cli::shared::IrCliOpt;
use forc_pkg::{self as pkg, fuel_core_not_running, DumpOpts, PackageManifestFile};
use forc_diagnostic::println_warning;
use forc_util::tx_utils::format_log_receipts;
use fuel_abi_types::abi::program::ProgramABI;
use fuel_core_client::client::FuelClient;
use fuel_tx::{ContractId, Transaction};
use fuels::{
    programs::calls::{traits::TransactionTuner, ScriptCall},
    types::{
        transaction::TxPolicies,
        transaction_builders::{BuildableTransaction, VariableOutputPolicy},
    },
};
use fuels_accounts::{provider::Provider, Account, ViewOnlyAccount};
use pkg::BuiltPackage;
use std::time::Duration;
use std::{path::PathBuf, str::FromStr};
use sway_core::BuildTarget;
use sway_core::{language::parsed::TreeType, IrCli};
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
    let wallet_mode = if command.default_signer || command.signing_key.is_some() {
        SignerSelectionMode::Manual
    } else {
        let password = prompt_forc_wallet_password()?;
        SignerSelectionMode::ForcWallet(password)
    };
    for built in built_pkgs_with_manifest {
        if built
            .descriptor
            .manifest_file
            .check_program_type(&[TreeType::Script])
            .is_ok()
        {
            let pkg_receipts = run_pkg(
                &command,
                &built.descriptor.manifest_file,
                &built,
                &wallet_mode,
            )
            .await?;
            receipts.push(pkg_receipts);
        }
    }

    Ok(receipts)
}

fn tx_policies_from_cmd(cmd: &cmd::Run) -> TxPolicies {
    let mut tx_policies = TxPolicies::default();
    if let Some(max_fee) = cmd.gas.max_fee {
        tx_policies = tx_policies.with_max_fee(max_fee);
    }
    if let Some(script_gas_limit) = cmd.gas.script_gas_limit {
        tx_policies = tx_policies.with_script_gas_limit(script_gas_limit);
    }
    tx_policies
}

pub async fn run_pkg(
    command: &cmd::Run,
    manifest: &PackageManifestFile,
    compiled: &BuiltPackage,
    signer_mode: &SignerSelectionMode,
) -> Result<RanScript> {
    let node_url = command.node.get_node_url(&manifest.network)?;
    let provider = Provider::connect(node_url.clone()).await?;
    let consensus_params = provider.consensus_parameters().await?;
    let tx_count = 1;
    let account = select_account(
        signer_mode,
        command.default_signer || command.unsigned,
        command.signing_key,
        &provider,
        tx_count,
    )
    .await?;

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

    let external_contracts = command
        .contract
        .as_ref()
        .into_iter()
        .flat_map(|contracts| contracts.iter())
        .map(|contract| {
            ContractId::from_str(contract)
                .map_err(|e| anyhow!("Failed to parse contract id: {}", e))
        })
        .collect::<Result<Vec<ContractId>>>()?;

    let script_binary = compiled.bytecode.bytes.clone();
    let call = ScriptCall {
        script_binary,
        encoded_args: Ok(script_data),
        inputs: vec![],
        outputs: vec![],
        external_contracts,
    };
    let tx_policies = tx_policies_from_cmd(command);
    let mut tb = call.transaction_builder(
        tx_policies,
        VariableOutputPolicy::EstimateMinimum,
        &consensus_params,
        call.inputs.clone(),
        &account,
    )?;

    account.add_witnesses(&mut tb)?;
    account.adjust_for_fee(&mut tb, 0).await?;

    let tx = tb.build(provider).await?;

    if command.dry_run {
        info!("{:?}", tx);
        Ok(RanScript { receipts: vec![] })
    } else {
        let program_abi = match &compiled.program_abi {
            sway_core::asm_generation::ProgramABI::Fuel(abi) => Some(abi),
            _ => None,
        };
        let receipts = try_send_tx(
            node_url.as_str(),
            &tx.into(),
            command.pretty_print,
            command.simulate,
            command.debug,
            program_abi,
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
    debug: bool,
    abi: Option<&ProgramABI>,
) -> Result<Vec<fuel_tx::Receipt>> {
    let client = FuelClient::new(node_url)?;

    match client.health().await {
        Ok(_) => timeout(
            Duration::from_millis(TX_SUBMIT_TIMEOUT_MS),
            send_tx(&client, tx, pretty_print, simulate, debug, abi),
        )
        .await
        .with_context(|| format!("timeout waiting for {tx:?} to be included in a block"))?,
        Err(_) => Err(fuel_core_not_running(node_url)),
    }
}

async fn send_tx(
    client: &FuelClient,
    tx: &Transaction,
    pretty_print: bool,
    simulate: bool,
    debug: bool,
    abi: Option<&ProgramABI>,
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
    if debug {
        start_debug_session(client, tx, abi).await?;
    }
    Ok(outputs)
}

/// Starts an interactive debugging session with the given transaction
async fn start_debug_session(
    fuel_client: &FuelClient,
    tx: &fuel_tx::Transaction,
    program_abi: Option<&ProgramABI>,
) -> Result<()> {
    // Create debugger instance from the existing fuel client
    let mut debugger = forc_debug::debugger::Debugger::from_client(fuel_client.clone())
        .await
        .map_err(|e| anyhow!("Failed to create debugger: {e}"))?;

    // Create temporary files for transaction and ABI (auto-cleaned when dropped)
    let mut tx_file = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .map_err(|e| anyhow!("Failed to create temp transaction file: {e}"))?;
    serde_json::to_writer_pretty(&mut tx_file, tx)
        .map_err(|e| anyhow!("Failed to write transaction to temp file: {e}"))?;

    let mut abi_file = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .map_err(|e| anyhow!("Failed to create temp ABI file: {e}"))?;

    let tx_cmd = if let Some(abi) = program_abi {
        serde_json::to_writer_pretty(&mut abi_file, &abi)
            .map_err(|e| anyhow!("Failed to write ABI to temp file: {e}"))?;

        // Prepare the start_tx command string for the CLI
        format!(
            "start_tx {} {}",
            tx_file.path().to_string_lossy(),
            abi_file.path().to_string_lossy()
        )
    } else {
        // Prepare the start_tx command string for the CLI
        format!("start_tx {}", tx_file.path().to_string_lossy())
    };

    // Start the interactive CLI session with the prepared command
    let mut cli = forc_debug::cli::Cli::new()
        .map_err(|e| anyhow!("Failed to create debug CLI interface: {e}"))?;
    cli.run(&mut debugger, Some(tx_cmd))
        .await
        .map_err(|e| anyhow!("Interactive debugging session failed: {e}"))?;

    Ok(())
}

fn build_opts_from_cmd(cmd: &cmd::Run) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.pkg.path.clone(),
            offline: cmd.pkg.offline,
            terse: cmd.pkg.terse,
            locked: cmd.pkg.locked,
            output_directory: cmd.pkg.output_directory.clone(),
            ipfs_node: cmd.pkg.ipfs_node.clone().unwrap_or_default(),
        },
        print: pkg::PrintOpts {
            ast: cmd.print.ast,
            dca_graph: cmd.print.dca_graph.clone(),
            dca_graph_url_format: cmd.print.dca_graph_url_format.clone(),
            asm: cmd.print.asm(),
            bytecode: cmd.print.bytecode,
            bytecode_spans: false,
            ir: cmd.print.ir(),
            reverse_order: cmd.print.reverse_order,
        },
        verify_ir: cmd
            .verify_ir
            .as_ref()
            .map_or(IrCli::default(), |opts| IrCliOpt::from(opts).0),
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify.json_abi,
            json_storage_slots: cmd.minify.json_storage_slots,
        },
        dump: DumpOpts::default(),
        build_target: BuildTarget::default(),
        build_profile: cmd.build_profile.build_profile.clone(),
        release: cmd.build_profile.release,
        error_on_warnings: cmd.build_profile.error_on_warnings,
        time_phases: cmd.print.time_phases,
        profile: cmd.print.profile,
        metrics_outfile: cmd.print.metrics_outfile.clone(),
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file.clone(),
        hex_outfile: cmd.build_output.hex_file.clone(),
        tests: false,
        member_filter: pkg::MemberFilter::only_scripts(),
        experimental: cmd.experimental.experimental.clone(),
        no_experimental: cmd.experimental.no_experimental.clone(),
        no_output: false,
    }
}
