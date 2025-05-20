mod call_function;
mod list_functions;
mod missing_contracts;
mod parser;
mod transfer;

use crate::{
    cmd,
    constants::DEFAULT_PRIVATE_KEY,
    op::call::{
        call_function::call_function, list_functions::list_contract_functions, transfer::transfer,
    },
    util::tx::{prompt_forc_wallet_password, select_local_wallet_account},
};
use anyhow::{anyhow, Result};
use either::Either;
use fuel_abi_types::abi::{program::ProgramABI, unified_program::UnifiedProgramABI};
use fuel_tx::Receipt;
use fuels::{
    accounts::{
        provider::Provider, signers::private_key::PrivateKeySigner, wallet::Wallet, ViewOnlyAccount,
    },
    crypto::SecretKey,
};
use fuels_core::types::{transaction::TxPolicies, AssetId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use sway_core;

/// Response returned from a contract call operation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CallResponse {
    pub tx_hash: String,
    pub result: Option<String>,
    pub logs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipts: Option<Vec<Receipt>>,
    #[serde(rename = "Script", skip_serializing_if = "Option::is_none")]
    pub script: Option<serde_json::Value>,
}

/// A command for calling a contract function.
pub async fn call(operation: cmd::call::Operation, cmd: cmd::Call) -> anyhow::Result<CallResponse> {
    let is_json_mode = matches!(cmd.output, cmd::call::OutputFormat::Json);
    let response = match operation {
        cmd::call::Operation::ListFunctions { contract_id, abi } => {
            if let cmd::call::OutputFormat::Json = cmd.output {
                return Err(anyhow!("JSON output is not supported for list functions"));
            }

            let abi_str = load_abi(&abi).await?;
            let parsed_abi: ProgramABI = serde_json::from_str(&abi_str)?;
            let unified_program_abi = UnifiedProgramABI::from_counterpart(&parsed_abi)?;

            list_contract_functions(
                &contract_id,
                &abi,
                &unified_program_abi,
                &mut std::io::stdout(),
            )?;

            CallResponse::default()
        }
        cmd::call::Operation::DirectTransfer {
            recipient,
            amount,
            asset_id,
        } => {
            let cmd::Call {
                node,
                caller,
                gas,
                verbosity,
                mut output,
                ..
            } = cmd;

            // Already validated that mode is ExecutionMode::Live
            let (wallet, tx_policies, base_asset_id) =
                setup_connection(&node, caller, &gas).await?;
            let asset_id = asset_id.unwrap_or(base_asset_id);

            transfer(
                &wallet,
                recipient,
                amount,
                asset_id,
                tx_policies,
                &node,
                verbosity,
                &mut output,
            )
            .await?
        }
        cmd::call::Operation::CallFunction {
            contract_id,
            abi,
            function,
            function_args,
        } => {
            // Call the function with required parameters
            call_function(contract_id, abi, function, function_args, cmd).await?
        }
    };

    // If using JSON output mode, explicitly print the response for potential parsing/piping
    if is_json_mode {
        println!("{}", serde_json::to_string_pretty(&response).unwrap());
    }

    Ok(response)
}

/// Sets up the connection to the node and initializes common parameters
async fn setup_connection(
    node: &crate::NodeTarget,
    caller: cmd::call::Caller,
    gas: &Option<forc_tx::Gas>,
) -> anyhow::Result<(Wallet, TxPolicies, AssetId)> {
    let node_url = node.get_node_url(&None)?;
    let provider = Provider::connect(node_url).await?;
    let wallet = get_wallet(caller.signing_key, caller.wallet, provider).await?;
    let provider = wallet.provider();
    let tx_policies = gas.as_ref().map(Into::into).unwrap_or_default();
    let consensus_parameters = provider.consensus_parameters().await?;
    let base_asset_id = consensus_parameters.base_asset_id();

    Ok((wallet, tx_policies, *base_asset_id))
}

/// Helper function to load ABI from file or URL
async fn load_abi(abi: &Either<std::path::PathBuf, url::Url>) -> anyhow::Result<String> {
    match abi {
        Either::Left(path) => std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read ABI file at {:?}: {}", path, e)),
        Either::Right(url) => {
            let response = reqwest::get(url.clone())
                .await
                .map_err(|e| anyhow!("Failed to fetch ABI from URL {}: {}", url, e))?;
            let bytes = response
                .bytes()
                .await
                .map_err(|e| anyhow!("Failed to read response body from URL {}: {}", url, e))?;
            String::from_utf8(bytes.to_vec())
                .map_err(|e| anyhow!("Failed to parse response as UTF-8 from URL {}: {}", url, e))
        }
    }
}

/// Get the wallet to use for the call - based on optionally provided signing key and wallet flag.
async fn get_wallet(
    signing_key: Option<SecretKey>,
    use_wallet: bool,
    provider: Provider,
) -> Result<Wallet> {
    match (signing_key, use_wallet) {
        (None, false) => {
            let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY).unwrap();
            let signer = PrivateKeySigner::new(secret_key);
            let wallet = Wallet::new(signer, provider);
            forc_tracing::println_warning(&format!(
                "No signing key or wallet flag provided. Using default signer: 0x{}",
                wallet.address().hash()
            ));
            Ok(wallet)
        }
        (Some(secret_key), false) => {
            let signer = PrivateKeySigner::new(secret_key);
            let wallet = Wallet::new(signer, provider);
            forc_tracing::println_warning(&format!(
                "Using account {} derived from signing key...",
                wallet.address().hash()
            ));
            Ok(wallet)
        }
        (None, true) => {
            let password = prompt_forc_wallet_password()?;
            let wallet = select_local_wallet_account(&password, &provider).await?;
            Ok(wallet)
        }
        (Some(secret_key), true) => {
            forc_tracing::println_warning(
                "Signing key is provided while requesting to use forc-wallet. Using signing key...",
            );
            let signer = PrivateKeySigner::new(secret_key);
            let wallet = Wallet::new(signer, provider);
            Ok(wallet)
        }
    }
}

/// Processes transaction receipts, logs, and displays transaction information
pub(crate) fn process_transaction_output(
    receipts: &[Receipt],
    tx_hash: &str,
    program_abi: &sway_core::asm_generation::ProgramABI,
    result: Option<String>,
    mode: &cmd::call::ExecutionMode,
    node: &crate::NodeTarget,
    verbosity: u8,
) -> Result<CallResponse> {
    // print receipts
    if verbosity >= 2 {
        let formatted_receipts = forc_util::tx_utils::format_log_receipts(receipts, true)?;
        forc_tracing::println_label_green("receipts:", &formatted_receipts);
    }

    let logs = receipts
        .iter()
        .filter_map(|receipt| match receipt {
            Receipt::LogData {
                rb,
                data: Some(data),
                ..
            } => forc_util::tx_utils::decode_log_data(&rb.to_string(), data, program_abi)
                .ok()
                .map(|decoded| decoded.value),
            _ => None,
        })
        .collect::<Vec<_>>();

    // display logs if verbosity is set
    if verbosity >= 1 && !logs.is_empty() {
        forc_tracing::println_green_bold("logs:");
        for log in logs.iter() {
            println!("  {:#}", log);
        }
    }

    // print tx hash and result
    forc_tracing::println_label_green("tx hash:", tx_hash);
    if let Some(result) = result.as_ref() {
        forc_tracing::println_label_green("result:", result);
    }

    // display transaction url if live mode
    if *mode == cmd::call::ExecutionMode::Live {
        if let Some(explorer_url) = node.get_explorer_url() {
            forc_tracing::println_label_green(
                "\nView transaction:",
                &format!("{}/tx/0x{}\n", explorer_url, tx_hash),
            );
        }
    }

    Ok(CallResponse {
        tx_hash: tx_hash.to_string(),
        result,
        receipts: if verbosity >= 2 {
            Some(receipts.to_vec())
        } else {
            None
        },
        script: None,
        logs,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use fuels::prelude::*;

    abigen!(Contract(
        name = "TestContract",
        abi = "forc-plugins/forc-client/tests/data/contract_with_types/contract_with_types-abi.json"
    ));

    pub async fn get_contract_instance() -> (TestContract<Wallet>, ContractId, Provider, SecretKey)
    {
        let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY).unwrap();
        let signer = PrivateKeySigner::new(secret_key);
        let coins = setup_single_asset_coins(signer.address(), AssetId::zeroed(), 1, 1_000_000);
        let provider = setup_test_provider(coins, vec![], None, None)
            .await
            .unwrap();
        let wallet = get_wallet(Some(secret_key), false, provider.clone())
            .await
            .unwrap();

        let id = Contract::load_from(
            "../../forc-plugins/forc-client/tests/data/contract_with_types/contract_with_types.bin",
            LoadConfiguration::default(),
        )
        .unwrap()
        .deploy(&wallet, TxPolicies::default())
        .await
        .unwrap()
        .contract_id;

        let instance = TestContract::new(id.clone(), wallet.clone());

        (instance, id.into(), provider, secret_key)
    }
}
