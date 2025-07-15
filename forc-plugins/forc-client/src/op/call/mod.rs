mod call_function;
pub mod list_functions;
mod missing_contracts;
mod parser;
pub mod trace;
mod transfer;

use crate::cmd::call::AbiSource;
use crate::{
    cmd,
    constants::DEFAULT_PRIVATE_KEY,
    op::call::{
        call_function::call_function, list_functions::list_contract_functions,
        trace::display_transaction_trace, transfer::transfer,
    },
    util::tx::{prompt_forc_wallet_password, select_local_wallet_account},
};
use anyhow::{anyhow, Result};
use fuel_abi_types::abi::{
    program::ProgramABI,
    unified_program::{UnifiedProgramABI, UnifiedTypeDeclaration},
};
use fuel_core_types::services::executor::TransactionExecutionStatus;
use fuel_tx::Receipt;
use fuels::{
    accounts::{
        provider::Provider, signers::private_key::PrivateKeySigner, wallet::Wallet, ViewOnlyAccount,
    },
    crypto::SecretKey,
};
use fuels_core::types::{transaction::TxPolicies, AssetId, ContractId};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

/// Response returned from a contract call operation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CallResponse {
    pub tx_hash: String,
    pub total_gas: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<Receipt>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trace_events: Vec<trace::TraceEvent>,
    #[serde(rename = "script", skip_serializing_if = "Option::is_none")]
    pub script_json: Option<serde_json::Value>,
}

/// A command for calling a contract function.
pub async fn call(operation: cmd::call::Operation, cmd: cmd::Call) -> anyhow::Result<CallResponse> {
    let is_json_mode = matches!(cmd.output, cmd::call::OutputFormat::Json);
    let response = match operation {
        cmd::call::Operation::ListFunctions { contract_id, abi } => {
            if let cmd::call::OutputFormat::Json = cmd.output {
                return Err(anyhow!("JSON output is not supported for list functions"));
            }

            let abi_map = create_abi_map(contract_id, &abi, cmd.contract_abis).await?;

            // Use the simplified list_contract_functions function
            list_contract_functions(&contract_id, &abi_map, &mut std::io::stdout())?;

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

/// Helper function to load ABI from file, URL, or raw string
async fn load_abi(abi: &AbiSource) -> anyhow::Result<String> {
    match abi {
        AbiSource::File(path) => std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read ABI file at {:?}: {}", path, e)),
        AbiSource::Url(url) => {
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
        AbiSource::String(json_str) => {
            // Validate that it's valid JSON
            serde_json::from_str::<serde_json::Value>(json_str)
                .map_err(|e| anyhow!("Invalid JSON in ABI string: {}", e))?;
            Ok(json_str.to_owned())
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
                wallet.address()
            ));
            Ok(wallet)
        }
        (Some(secret_key), false) => {
            let signer = PrivateKeySigner::new(secret_key);
            let wallet = Wallet::new(signer, provider);
            forc_tracing::println_warning(&format!(
                "Using account {} derived from signing key...",
                wallet.address()
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

#[derive(Debug, Clone)]
pub struct Abi {
    source: AbiSource,
    program: ProgramABI,
    unified: UnifiedProgramABI,
    // TODO: required for vm interpreter step through
    // ↳ gh issue: https://github.com/FuelLabs/sway/issues/7197
    #[allow(dead_code)]
    type_lookup: HashMap<usize, UnifiedTypeDeclaration>,
}

impl FromStr for Abi {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let program: ProgramABI =
            serde_json::from_str(s).map_err(|err| format!("failed to parse ABI: {}", err))?;

        let unified = UnifiedProgramABI::from_counterpart(&program)
            .map_err(|err| format!("conversion to unified ABI format failed: {}", err))?;

        let type_lookup = unified
            .types
            .iter()
            .map(|decl| (decl.type_id, decl.clone()))
            .collect::<HashMap<_, _>>();

        Ok(Self {
            source: AbiSource::String(s.to_string()),
            program,
            unified,
            type_lookup,
        })
    }
}

/// Displays transaction information
pub(crate) fn display_tx_info(
    tx_hash: String,
    result: Option<String>,
    mode: &cmd::call::ExecutionMode,
    node: &crate::NodeTarget,
) {
    // print tx hash and result
    forc_tracing::println_label_green("tx hash:", &tx_hash);
    if let Some(ref result) = result {
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
}

/// Prints receipts and trace to the writer based on verbosity level
pub(crate) fn display_detailed_call_info(
    tx: &TransactionExecutionStatus,
    script_json: &serde_json::Value,
    abis: &HashMap<ContractId, Abi>,
    verbosity: u8,
    writer: &mut impl std::io::Write,
    trace_events: &[trace::TraceEvent],
    labels: &HashMap<ContractId, String>,
) -> Result<()> {
    if verbosity >= 4 {
        forc_tracing::println_label_green(
            "transaction script:\n",
            &serde_json::to_string_pretty(script_json).unwrap(),
        );
    }
    if verbosity >= 3 {
        let formatted_receipts =
            forc_util::tx_utils::format_log_receipts(tx.result.receipts(), true)
                .map_err(|e| anyhow!("Failed to format receipts: {}", e))?;
        forc_tracing::println_label_green("receipts:", &formatted_receipts);
    }
    if verbosity >= 2 {
        display_transaction_trace(*tx.result.total_gas(), trace_events, labels, writer)
            .map_err(|e| anyhow!("Failed to display transaction trace: {e}"))?;
    }
    if verbosity >= 1 {
        let logs = tx
            .result
            .receipts()
            .iter()
            .filter_map(|receipt| match receipt {
                Receipt::LogData {
                    id,
                    rb,
                    data: Some(data),
                    ..
                } => {
                    let default_program_abi = ProgramABI::default();
                    let program_abi = abis
                        .get(id)
                        .map(|abi| &abi.program)
                        .unwrap_or(&default_program_abi);
                    forc_util::tx_utils::decode_fuel_vm_log_data(&rb.to_string(), data, program_abi)
                        .ok()
                        .map(|decoded| decoded.value)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        // print logs if there are any
        if !logs.is_empty() {
            forc_tracing::println_green_bold("logs:");
            for log in logs.iter() {
                writeln!(writer, "  {:#}", log)?;
            }
        }
    }
    Ok(())
}

/// Create a HashMap of contract ABIs from a main ABI and optional additional contract ABIs
/// This is a reusable function for both call_function and list_functions operations
pub async fn create_abi_map(
    main_contract_id: ContractId,
    main_abi: &AbiSource,
    additional_contract_abis: Option<Vec<(ContractId, AbiSource)>>,
) -> anyhow::Result<HashMap<ContractId, Abi>> {
    // Load main ABI
    let main_abi_str = load_abi(main_abi).await?;
    let main_abi =
        Abi::from_str(&main_abi_str).map_err(|e| anyhow!("Failed to parse main ABI: {}", e))?;

    // Start with main contract ABI
    let mut abi_map = HashMap::from([(main_contract_id, main_abi)]);

    // Load additional contract ABIs if provided
    if let Some(contract_abis) = additional_contract_abis {
        for (contract_id, abi_path) in contract_abis {
            match load_abi(&abi_path).await {
                Ok(abi_str) => match Abi::from_str(&abi_str) {
                    Ok(additional_abi) => {
                        abi_map.insert(contract_id, additional_abi);
                        forc_tracing::println_action_green(
                            "Loaded additional ABI for contract",
                            &format!("0x{}", contract_id),
                        );
                    }
                    Err(e) => {
                        forc_tracing::println_warning(&format!(
                            "Failed to parse ABI for contract 0x{}: {}",
                            contract_id, e
                        ));
                    }
                },
                Err(e) => {
                    forc_tracing::println_warning(&format!(
                        "Failed to load ABI for contract 0x{}: {}",
                        contract_id, e
                    ));
                }
            }
        }
    }

    Ok(abi_map)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use fuels::prelude::*;

    abigen!(Contract(
        name = "TestContract",
        abi = "forc-plugins/forc-client/test/data/contract_with_types/contract_with_types-abi.json"
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
            "../../forc-plugins/forc-client/test/data/contract_with_types/contract_with_types.bin",
            LoadConfiguration::default(),
        )
        .unwrap()
        .deploy(&wallet, TxPolicies::default())
        .await
        .unwrap()
        .contract_id;

        let instance = TestContract::new(id, wallet.clone());

        (instance, id, provider, secret_key)
    }
}
