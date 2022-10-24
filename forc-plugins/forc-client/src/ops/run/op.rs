use anyhow::{anyhow, bail, Result};
use forc_pkg::{fuel_core_not_running, BuildOptions, PackageManifestFile};
use fuel_gql_client::client::FuelClient;
use fuel_tx::{ContractId, Transaction, TransactionBuilder};
use futures::TryFutureExt;
use std::{path::PathBuf, str::FromStr};
use sway_core::language::parsed::TreeType;
use tracing::info;

use crate::ops::tx_util::{TransactionBuilderExt, TxParameters};

use super::cmd::RunCommand;

pub const NODE_URL: &str = "http://127.0.0.1:4000";

pub async fn run(command: RunCommand) -> Result<Vec<fuel_tx::Receipt>> {
    let path_dir = if let Some(path) = &command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };
    let manifest = PackageManifestFile::from_dir(&path_dir)?;
    manifest.check_program_type(vec![TreeType::Script])?;

    let input_data = &command.data.unwrap_or_else(|| "".into());
    let data = format_hex_data(input_data);
    let script_data = hex::decode(data).expect("Invalid hex");

    let node_url = command.node_url.unwrap_or_else(|| match &manifest.network {
        Some(network) => network.url.to_owned(),
        None => NODE_URL.to_owned(),
    });
    let client = FuelClient::new(&node_url)?;

    let build_options = BuildOptions {
        path: command.path,
        print_ast: command.print_ast,
        print_finalized_asm: command.print_finalized_asm,
        print_intermediate_asm: command.print_intermediate_asm,
        print_ir: command.print_ir,
        binary_outfile: command.binary_outfile,
        debug_outfile: command.debug_outfile,
        offline_mode: false,
        terse_mode: command.terse_mode,
        output_directory: command.output_directory,
        minify_json_abi: command.minify_json_abi,
        minify_json_storage_slots: command.minify_json_storage_slots,
        locked: command.locked,
        build_profile: None,
        release: false,
        time_phases: command.time_phases,
    };
    let compiled = forc_pkg::build_with_options(build_options)?;

    let contract_ids: Vec<ContractId> = command
        .contract
        .unwrap_or_default()
        .iter()
        .map(|contract_id| ContractId::from_str(contract_id).unwrap())
        .collect();
    let tx = TransactionBuilder::script(compiled.bytecode, script_data)
        .params(TxParameters::new(command.gas_limit, command.gas_price))
        .add_contracts(contract_ids)
        .finalize_signed(client.clone(), command.unsigned, command.signing_key)
        .await?;

    if command.dry_run {
        info!("{:?}", tx);
        Ok(vec![])
    } else {
        try_send_tx(&node_url, &tx, command.pretty_print, command.simulate).await
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
        Ok(_) => send_tx(&client, tx, pretty_print, simulate).await,
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
                .submit(tx)
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
    let mut receipt_to_json_array = serde_json::to_value(&receipts)?;
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
