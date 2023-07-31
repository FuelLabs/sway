mod encode;
use crate::{
    cmd,
    util::{
        pkg::built_pkgs,
        tx::{TransactionBuilderExt, WalletSelectionMode, TX_SUBMIT_TIMEOUT_MS},
    },
};
use anyhow::{anyhow, bail, Context, Result};
use forc_pkg::{self as pkg, fuel_core_not_running, PackageManifestFile};
use forc_util::tx_utils::format_log_receipts;
use fuel_core_client::client::FuelClient;
use fuel_crypto::SecretKey;
use fuel_tx::{AssetId, ContractId, Transaction, TransactionBuilder};
use fuels_accounts::{predicate::Predicate, provider::Provider, wallet::{Wallet, WalletUnlocked}, Account};
use fuels_core::types::transaction::TxParameters;
use pkg::BuiltPackage;
use std::{fs, time::Duration};
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
    let mut receipts = Vec::new();
    let curr_dir = if let Some(path) = &command.pkg.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };
    let build_opts = build_opts_from_cmd(&command);
    let built_pkgs_with_manifest = built_pkgs(&curr_dir, build_opts)?;
    for built in built_pkgs_with_manifest {
        let built_manifest_file = &built.descriptor.manifest_file;
        if built_manifest_file
            .check_program_type(vec![TreeType::Script])
            .is_ok()
        {
            println!("running script");
            let is_predicate = false;
            let script_pkg_receipts = run_script_pkg(
                &command,
                &built.descriptor.manifest_file,
                &built,
                is_predicate,
            )
            .await?;
            receipts.push(script_pkg_receipts);
        } else if built_manifest_file
            .check_program_type(vec![TreeType::Predicate])
            .is_ok()
        {
            println!("spending predicate");
            let is_predicate = true;
            let script_pkg_receipts = run_script_pkg(
                &command,
                &built.descriptor.manifest_file,
                &built,
                is_predicate,
            )
            .await?;
            receipts.push(script_pkg_receipts);
        }
    }

    Ok(receipts)
}

pub async fn run_script_pkg(
    command: &cmd::Run,
    manifest: &PackageManifestFile,
    compiled: &BuiltPackage,
    is_predicate: bool,
) -> Result<RanScript> {
    let script_data = match (&command.data, &command.args, is_predicate) {
        (None, Some(args), false) => {
            let minify_json_abi = true;
            let package_json_abi = compiled
                .json_abi_string(minify_json_abi)?
                .ok_or_else(|| anyhow::anyhow!("Missing json abi string"))?;
            let main_arg_handler = ScriptCallHandler::from_json_abi_str(&package_json_abi)?;
            let args = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();
            let unresolved_bytes = main_arg_handler.encode_arguments(args.as_slice())?;
            Some(unresolved_bytes.resolve(0))
        }
        (Some(_), Some(_), _) => {
            bail!("Both --args and --data provided, must choose one.")
        }
        (_, _, true) => None,
        _ => {
            let input_data = command.data.as_deref().unwrap_or("");
            let data = input_data.strip_prefix("0x").unwrap_or(input_data);
            Some(hex::decode(data).expect("Invalid hex"))
        }
    };
    let node_url = command
        .node_url
        .as_deref()
        .or_else(|| manifest.network.as_ref().map(|nw| &nw.url[..]))
        .unwrap_or(crate::default::NODE_URL);

    let client = FuelClient::new(node_url)?;
    if let Some(script_data) = script_data {
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

        let tx = TransactionBuilder::script(compiled.bytecode.bytes.clone(), script_data)
            .gas_limit(command.gas.limit)
            .gas_price(command.gas.price)
            .maturity(command.maturity.maturity.into())
            .add_contracts(contract_ids)
            .finalize_signed(
                client.clone(),
                command.unsigned,
                command.signing_key,
                wallet_mode,
            )
            .await?;
        if command.dry_run {
            info!("{:?}", tx);
            Ok(RanScript { receipts: vec![] })
        } else {
            let receipts =
                try_send_tx(node_url, &tx.into(), command.pretty_print, command.simulate).await?;
            Ok(RanScript { receipts })
        }
    } else {
        let minify_json_abi = true;
        let package_json_abi = compiled
            .json_abi_string(minify_json_abi)?
           .ok_or_else(|| anyhow::anyhow!("Missing json abi string"))?;
        let main_arg_handler = ScriptCallHandler::from_json_abi_str(&package_json_abi)?;
        let args = command.args.clone().unwrap_or_default();
        let args = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();
        let unresolved_bytes = main_arg_handler.encode_arguments(args.as_slice())?;

        let spend_amount = command
            .spend_amount
            .as_ref()
            .ok_or_else(|| anyhow!("To spend a predicate, --spend-amount should be provided"))?;
        let receive_address = command
            .receive_address
            .as_ref()
            .ok_or_else(|| anyhow!("To spend a predicate, --receive-address should be provided"))?;
        println!("receive-address {receive_address}");
        println!("spend_amount {spend_amount}");
        let asset_id = AssetId::default();
        let cons_params = client.chain_info().await?.consensus_parameters.into();
        let tx_params = TxParameters::new(
            command.gas.price,
            command.gas.limit,
            command.maturity.maturity,
        );
        let provider = Provider::new(client, cons_params);

        let private_key = SecretKey::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c")?;
        let test_account = WalletUnlocked::new_from_private_key(private_key, Some(provider.clone()));

        println!("{:?}", compiled.bytecode.bytes.clone());
        let predicate =
            Predicate::from_code(compiled.bytecode.bytes.clone()).with_provider(provider).with_data(unresolved_bytes);
        println!("{predicate:?}");
        test_account.transfer(predicate.address(), 100, asset_id, tx_params).await?;

        if command.dry_run {
            Ok(RanScript { receipts: vec![] })
        } else {
            let (_, receipts) = predicate
                .transfer(receive_address, *spend_amount, asset_id, tx_params)
                .await?;
            Ok(RanScript { receipts })
        }
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
    use fuels_accounts::provider::ClientExt;
    let outputs = {
        if !simulate {
            let (_, receipts) = client.submit_and_await_commit_with_receipts(tx).await?;
            if let Some(receipts) = receipts {
                Ok(receipts)
            } else {
                bail!("The `receipts` during `send_tx` is empty")
            }
        } else {
            client.dry_run(tx).await
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

fn build_opts_from_cmd(cmd: &cmd::Run) -> pkg::BuildOpts {
    let mut filter = pkg::MemberFilter::only_scripts();
    filter.build_predicates = true;
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
            finalized_asm: cmd.print.finalized_asm,
            intermediate_asm: cmd.print.intermediate_asm,
            ir: cmd.print.ir,
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
        member_filter: filter,
    }
}
