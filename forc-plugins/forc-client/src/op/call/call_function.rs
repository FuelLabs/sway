use crate::{
    cmd::{self, call::FuncType},
    op::call::{
        missing_contracts::determine_missing_contracts,
        parser::{param_type_val_to_token, token_to_string},
        Abi, CallData, CallResponse, Either,
    },
};
use anyhow::{anyhow, bail, Result};
use fuel_abi_types::abi::unified_program::UnifiedProgramABI;
use fuels::{
    accounts::ViewOnlyAccount,
    programs::calls::{
        receipt_parser::ReceiptParser,
        traits::{ContractDependencyConfigurator, TransactionTuner},
        ContractCall,
    },
    types::tx_status::TxStatus,
};
use fuels_core::{
    codec::{
        encode_fn_selector, log_formatters_lookup, ABIDecoder, ABIEncoder, DecoderConfig,
        EncoderConfig, ErrorDetails, LogDecoder,
    },
    types::{
        bech32::Bech32ContractId,
        param_types::ParamType,
        transaction::Transaction,
        transaction_builders::{BuildableTransaction, ScriptBuildStrategy, VariableOutputPolicy},
        ContractId,
    },
};
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use url::Url;

/// Calls a contract function with the given parameters
pub async fn call_function(
    contract_id: ContractId,
    abi: Either<PathBuf, Url>,
    function: FuncType,
    function_args: Vec<String>,
    cmd: cmd::Call,
) -> Result<CallResponse> {
    let cmd::Call {
        node,
        mode,
        caller,
        call_parameters,
        gas,
        mut output,
        external_contracts,
        ..
    } = cmd;

    // Load ABI (already provided in the operation)
    let abi_str = super::load_abi(&abi).await?;
    let abi = Abi::from_str(&abi_str).map_err(|e| anyhow!("Failed to parse ABI: {}", e))?;

    let cmd::call::FuncType::Selector(selector) = function;

    let (encoded_data, output_param) =
        prepare_contract_call_data(&abi.unified, &selector, &function_args)?;

    // Setup connection to node
    let (wallet, tx_policies, base_asset_id) = super::setup_connection(&node, caller, &gas).await?;
    let call_parameters = cmd::call::CallParametersOpts {
        asset_id: call_parameters.asset_id.or(Some(base_asset_id)),
        ..call_parameters
    };

    // Create the contract call
    let call = ContractCall {
        contract_id: contract_id.into(),
        encoded_selector: encode_fn_selector(&selector),
        encoded_args: Ok(encoded_data),
        call_parameters: call_parameters.clone().into(),
        external_contracts: vec![], // set below
        output_param: output_param.clone(),
        is_payable: call_parameters.amount > 0,
        custom_assets: Default::default(),
        inputs: Vec::new(),
        outputs: Vec::new(),
    };

    // Setup variable output policy and log decoder
    let variable_output_policy = VariableOutputPolicy::Exactly(call_parameters.amount as usize);
    let error_codes = abi
        .unified
        .error_codes
        .as_ref()
        .map_or(HashMap::new(), |error_codes| {
            error_codes
                .iter()
                .map(|(revert_code, error_details)| {
                    (
                        *revert_code,
                        ErrorDetails::new(
                            error_details.pos.pkg.clone(),
                            error_details.pos.file.clone(),
                            error_details.pos.line,
                            error_details.pos.column,
                            error_details.log_id.clone(),
                            error_details.msg.clone(),
                        ),
                    )
                })
                .collect()
        });
    let log_decoder = LogDecoder::new(log_formatters_lookup(vec![], contract_id), error_codes);

    // Get external contracts (either provided or auto-detected)
    let external_contracts = match external_contracts {
        Some(external_contracts) => external_contracts
            .iter()
            .map(|addr| Bech32ContractId::from(*addr))
            .collect(),
        None => {
            // Automatically retrieve missing contract addresses from the call
            let external_contracts = determine_missing_contracts(
                &call,
                wallet.provider(),
                &tx_policies,
                &variable_output_policy,
                &log_decoder,
                &wallet,
            )
            .await?;
            if !external_contracts.is_empty() {
                forc_tracing::println_warning(
                    "Automatically provided external contract addresses with call (max 10):",
                );
                external_contracts.iter().for_each(|addr| {
                    forc_tracing::println_warning(&format!("- 0x{}", ContractId::from(addr)));
                });
            }
            external_contracts
        }
    };

    // Execute the call based on execution mode
    let chain_id = wallet.provider().consensus_parameters().await?.chain_id();
    let tb = call
        .clone()
        .with_external_contracts(external_contracts)
        .transaction_builder(tx_policies, variable_output_policy, &wallet)
        .await
        .map_err(|e| anyhow!("Failed to initialize transaction builder: {e}"))?;
    let (tx, tx_status) = match mode {
        cmd::call::ExecutionMode::DryRun => {
            let tx = call
                .build_tx(tb, &wallet)
                .await
                .map_err(|e| anyhow!("Failed to build transaction: {e}"))?;
            let tx_status = wallet
                .provider()
                .dry_run(tx.clone())
                .await
                .map_err(|e| anyhow!("Failed to dry run transaction: {e}"))?;
            (tx, tx_status)
        }
        cmd::call::ExecutionMode::Simulate => {
            let tb = tb.with_build_strategy(ScriptBuildStrategy::StateReadOnly);
            let tx = call
                .build_tx(tb, &wallet)
                .await
                .map_err(|e| anyhow!("Failed to build transaction: {e}"))?;
            let gas_price = gas.map(|g| g.price).unwrap_or(Some(0));
            let tx_status = wallet
                .provider()
                .dry_run_opt(tx.clone(), false, gas_price, None)
                .await
                .map_err(|e| anyhow!("Failed to simulate transaction: {e}"))?;
            (tx, tx_status)
        }
        cmd::call::ExecutionMode::Live => {
            forc_tracing::println_action_green(
                "Sending transaction with wallet",
                &format!("0x{}", wallet.address().hash()),
            );
            let tx = call
                .build_tx(tb, &wallet)
                .await
                .map_err(|e| anyhow!("Failed to build transaction: {e}"))?;
            let tx_status = wallet
                .provider()
                .send_transaction_and_await_commit(tx.clone())
                .await
                .map_err(|e| anyhow!("Failed to send transaction: {e}"))?;
            (tx, tx_status)
        }
    };
    let tx_hash = tx.id(chain_id);
    let fuel_tx::Transaction::Script(script) = tx.into() else {
        return Err(anyhow!("Transaction is not a script"));
    };

    // Display the script JSON when verbosity level is 2 or higher (vv)
    let script_json = if cmd.verbosity >= 2 {
        let script_json = serde_json::to_value(&script).unwrap();
        forc_tracing::println_label_green(
            "transaction script:\n",
            &serde_json::to_string_pretty(&script_json).unwrap(),
        );
        Some(script_json)
    } else {
        None
    };

    let abi_map = HashMap::from([(contract_id, abi)]);

    // Process transaction results
    let receipts = match tx_status.clone().take_receipts_checked(Some(&log_decoder)) {
        Ok(receipts) => receipts,
        Err(e) => {
            // Print receipts when an error occurs and verbosity is high enough
            super::print_receipts_and_trace(
                tx_status.total_gas(),
                &tx_status.clone().take_receipts(),
                cmd.verbosity,
                &abi_map,
                &mut output,
            )?;
            match tx_status {
                TxStatus::Failure(e) | TxStatus::PreconfirmationFailure(e) => {
                    bail!("Failed to process transaction; reason: {:#?}", e.reason);
                }
                _ => bail!("Failed to process transaction: {:#?}", e),
            }
        }
    };

    // Parse the result based on output format
    let mut receipt_parser = ReceiptParser::new(&receipts, DecoderConfig::default());
    let result = match output {
        cmd::call::OutputFormat::Default | cmd::call::OutputFormat::Json => {
            let data = receipt_parser
                .extract_contract_call_data(contract_id)
                .ok_or(anyhow!("Failed to extract contract call data"))?;
            ABIDecoder::default()
                .decode_as_debug_str(&output_param, data.as_slice())
                .map_err(|e| anyhow!("Failed to decode as debug string: {e}"))?
        }
        cmd::call::OutputFormat::Raw => {
            let token = receipt_parser
                .parse_call(&Bech32ContractId::from(contract_id), &output_param)
                .map_err(|e| anyhow!("Failed to parse call data: {e}"))?;
            token_to_string(&token)
                .map_err(|e| anyhow!("Failed to convert token to string: {e}"))?
        }
    };

    // Process and return the final output
    let mut call_response = super::process_transaction_output(
        tx_status,
        &tx_hash.to_string(),
        &mode,
        &node,
        cmd.verbosity,
        &mut output,
        Some(CallData {
            contract_id,
            abis: abi_map,
            result,
        }),
    )?;
    if cmd.verbosity >= 2 {
        call_response.script_json = script_json;
    }

    Ok(call_response)
}

fn prepare_contract_call_data(
    unified_program_abi: &UnifiedProgramABI,
    selector: &str,
    function_args: &[String],
) -> Result<(Vec<u8>, ParamType)> {
    let type_lookup = unified_program_abi
        .types
        .iter()
        .map(|decl| (decl.type_id, decl.clone()))
        .collect::<HashMap<_, _>>();

    // Find the function in the ABI
    let abi_func = unified_program_abi
        .functions
        .iter()
        .find(|f| f.name == selector)
        .cloned()
        .ok_or_else(|| anyhow!("Function '{selector}' not found in ABI"))?;

    // Validate number of arguments
    if abi_func.inputs.len() != function_args.len() {
        bail!(
            "Argument count mismatch for '{selector}': expected {}, got {}",
            abi_func.inputs.len(),
            function_args.len()
        );
    }

    // Parse function arguments to tokens
    let tokens = abi_func
        .inputs
        .iter()
        .zip(function_args)
        .map(|(type_application, arg)| {
            let param_type =
                ParamType::try_from_type_application(type_application, &type_lookup)
                    .map_err(|e| anyhow!("Failed to convert input type application: {e}"))?;
            param_type_val_to_token(&param_type, arg)
        })
        .collect::<Result<Vec<_>>>()?;

    // Get output parameter type
    let output_param = ParamType::try_from_type_application(&abi_func.output, &type_lookup)
        .map_err(|e| anyhow!("Failed to convert output type: {e}"))?;

    // Encode function arguments
    let abi_encoder = ABIEncoder::new(EncoderConfig::default());
    let encoded_data = abi_encoder
        .encode(&tokens)
        .map_err(|e| anyhow!("Failed to encode function arguments: {e}"))?;

    Ok((encoded_data, output_param))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        cmd,
        op::call::{call, get_wallet, PrivateKeySigner},
    };
    use fuels::{crypto::SecretKey, prelude::*};

    fn get_contract_call_cmd(
        id: ContractId,
        node_url: &str,
        secret_key: SecretKey,
        selector: &str,
        args: Vec<&str>,
    ) -> cmd::Call {
        cmd::Call {
            address: (*id).into(),
            abi: Some(Either::Left(std::path::PathBuf::from(
                "../../forc-plugins/forc-client/test/data/contract_with_types/contract_with_types-abi.json",
            ))),
            function: Some(selector.to_string()),
            function_args: args.into_iter().map(String::from).collect(),
            node: crate::NodeTarget { node_url: Some(node_url.to_string()), ..Default::default() },
            caller: cmd::call::Caller { signing_key: Some(secret_key), wallet: false },
            call_parameters: Default::default(),
            mode: cmd::call::ExecutionMode::DryRun,
            gas: None,
            external_contracts: None,
            output: cmd::call::OutputFormat::Raw,
            list_functions: false,
            verbosity: 0,
        }
    }

    abigen!(Contract(
        name = "TestContract",
        abi = "forc-plugins/forc-client/test/data/contract_with_types/contract_with_types-abi.json"
    ));

    pub async fn get_contract_instance() -> (TestContract<Wallet>, ContractId, Provider, SecretKey)
    {
        let secret_key = SecretKey::random(&mut rand::thread_rng());
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

        let instance = TestContract::new(id.clone(), wallet.clone());

        (instance, id.into(), provider, secret_key)
    }

    #[tokio::test]
    async fn contract_call_with_abi() {
        let (_, id, provider, secret_key) = get_contract_instance().await;
        let node_url = provider.url();

        // test_empty_no_return
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_empty_no_return", vec![]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "()");

        // test_empty
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_empty", vec![]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "()");

        // test_unit
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_unit", vec!["()"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "()");

        // test_u8
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_u8", vec!["255"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "255");

        // test_u16
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_u16", vec!["65535"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "65535");

        // test_u32
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_u32", vec!["4294967295"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "4294967295"
        );

        // test_u64
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_u64",
            vec!["18446744073709551615"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "18446744073709551615"
        );

        // test_u128
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_u128",
            vec!["340282366920938463463374607431768211455"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "340282366920938463463374607431768211455"
        );

        // test_u256
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_u256",
            vec!["115792089237316195423570985008687907853269984665640564039457584007913129639935"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        );

        // test b256
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_b256",
            vec!["0000000000000000000000000000000000000000000000000000000000000042"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000000042"
        );

        // test_b256 - fails if 0x prefix provided since it extracts input as an external contract; we don't want to do this so explicitly provide the external contract as empty
        let mut cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_b256",
            vec!["0x0000000000000000000000000000000000000000000000000000000000000042"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        cmd.external_contracts = Some(vec![]);
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000000042"
        );

        // test_bytes
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_bytes", vec!["0x42"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "0x42");

        // test bytes without 0x prefix
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_bytes", vec!["42"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "0x42");

        // test_str
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_str", vec!["fuel"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "fuel");

        // test str array
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_str_array",
            vec!["fuel rocks"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "fuel rocks"
        );

        // test str array - fails if length mismatch
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_str_array", vec!["fuel"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap_err().to_string(),
            "string array length mismatch: expected 10, got 4"
        );

        // test str slice
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_str_slice",
            vec!["fuel rocks 42"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "fuel rocks 42"
        );

        // test tuple
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_tuple", vec!["(42, true)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(42, true)"
        );

        // test array
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_array",
            vec!["[42, 42, 42, 42, 42, 42, 42, 42, 42, 42]"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "[42, 42, 42, 42, 42, 42, 42, 42, 42, 42]"
        );

        // test_array - fails if different types
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_array", vec!["[42, true]"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap_err().to_string(),
            "failed to parse u64 value: true"
        );

        // test_array - succeeds if length not matched!?
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_array", vec!["[42, 42]"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert!(call(operation, cmd)
            .await
            .unwrap()
            .result
            .unwrap()
            .starts_with("[42, 42, 0,"));

        // test_vector
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_vector", vec!["[42, 42]"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "[42, 42]"
        );

        // test_vector - fails if different types
        let cmd =
            get_contract_call_cmd(id, node_url, secret_key, "test_vector", vec!["[42, true]"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap_err().to_string(),
            "failed to parse u64 value: true"
        );

        // test_struct - Identity { name: str[2], id: u64 }
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_struct", vec!["{fu, 42}"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "{fu, 42}"
        );

        // test_struct - fails if incorrect inner attribute length
        let cmd =
            get_contract_call_cmd(id, node_url, secret_key, "test_struct", vec!["{fuel, 42}"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap_err().to_string(),
            "string array length mismatch: expected 2, got 4"
        );

        // test_struct - succeeds if missing inner final attribute; default value is used
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_struct", vec!["{fu}"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "{fu, 0}"
        );

        // test_struct - succeeds to use default values for all attributes if missing
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_struct", vec!["{}"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "{\0\0, 0}"
        );

        // test_enum
        let cmd =
            get_contract_call_cmd(id, node_url, secret_key, "test_enum", vec!["(Active:true)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(Active:true)"
        );

        // test_enum - succeeds if using index
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_enum", vec!["(1:56)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(Pending:56)"
        );

        // test_enum - fails if variant not found
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_enum", vec!["(A:true)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap_err().to_string(),
            "failed to find index of variant: A"
        );

        // test_enum - fails if variant value incorrect
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_enum", vec!["(Active:3)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap_err().to_string(),
            "failed to parse `Active` variant enum value: 3"
        );

        // test_enum - fails if variant value is missing
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_enum", vec!["(Active:)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap_err().to_string(),
            "enum must have exactly two parts `(variant:value)`: (Active:)"
        );

        // test_option - encoded like an enum
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_option", vec!["(0:())"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(None:())"
        );

        // test_option - encoded like an enum; none value ignored
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_option", vec!["(0:42)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(None:())"
        );

        // test_option - encoded like an enum; some value
        let cmd = get_contract_call_cmd(id, node_url, secret_key, "test_option", vec!["(1:42)"]);
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(Some:42)"
        );
    }

    #[tokio::test]
    async fn contract_call_with_abi_complex() {
        let (_, id, provider, secret_key) = get_contract_instance().await;
        let node_url = provider.url();

        // test_complex_struct
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_struct_with_generic",
            vec!["{42, fuel}"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "{42, fuel}"
        );

        // test_enum_with_generic
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_enum_with_generic",
            vec!["(value:32)"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(value:32)"
        );

        // test_enum_with_complex_generic
        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_enum_with_complex_generic",
            vec!["(value:{42, fuel})"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(value:{42, fuel})"
        );

        let cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "test_enum_with_complex_generic",
            vec!["(container:{{42, fuel}, fuel})"],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(
            call(operation, cmd).await.unwrap().result.unwrap(),
            "(container:{{42, fuel}, fuel})"
        );
    }

    #[tokio::test]
    async fn contract_value_forwarding() {
        let (_, id, provider, secret_key) = get_contract_instance().await;
        let node_url = provider.url();

        let consensus_parameters = provider.consensus_parameters().await.unwrap();
        let base_asset_id = consensus_parameters.base_asset_id();
        let get_recipient_balance = |addr: Bech32Address, provider: Provider| async move {
            provider
                .get_asset_balance(&addr, *base_asset_id)
                .await
                .unwrap()
        };
        let get_contract_balance = |id: ContractId, provider: Provider| async move {
            provider
                .get_contract_asset_balance(&Bech32ContractId::from(id), *base_asset_id)
                .await
                .unwrap()
        };

        // contract call transfer funds to another contract
        let (_, id_2, _, _) = get_contract_instance().await;
        let (amount, asset_id, recipient) = (
            "1",
            &format!("{{0x{}}}", base_asset_id),
            &format!("(ContractId:{{0x{}}})", id_2),
        );
        let mut cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "transfer",
            vec![amount, asset_id, recipient],
        );
        let operation = cmd.validate_and_get_operation().unwrap();
        cmd.call_parameters = cmd::call::CallParametersOpts {
            amount: amount.parse::<u64>().unwrap(),
            asset_id: Some(*base_asset_id),
            gas_forwarded: None,
        };
        // validate balance is unchanged (dry-run)
        assert_eq!(
            call(operation.clone(), cmd.clone())
                .await
                .unwrap()
                .result
                .unwrap(),
            "()"
        );
        assert_eq!(get_contract_balance(id_2, provider.clone()).await, 0);
        cmd.mode = cmd::call::ExecutionMode::Live;
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "()");
        assert_eq!(get_contract_balance(id_2, provider.clone()).await, 1);
        assert_eq!(get_contract_balance(id, provider.clone()).await, 1);

        // contract call transfer funds to another address
        let random_wallet = Wallet::random(&mut rand::thread_rng(), provider.clone());
        let (amount, asset_id, recipient) = (
            "2",
            &format!("{{0x{}}}", base_asset_id),
            &format!("(Address:{{0x{}}})", random_wallet.address().hash()),
        );
        let mut cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "transfer",
            vec![amount, asset_id, recipient],
        );
        cmd.call_parameters = cmd::call::CallParametersOpts {
            amount: amount.parse::<u64>().unwrap(),
            asset_id: Some(*base_asset_id),
            gas_forwarded: None,
        };
        cmd.mode = cmd::call::ExecutionMode::Live;
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "()");
        assert_eq!(
            get_recipient_balance(random_wallet.address().clone(), provider.clone()).await,
            2
        );
        assert_eq!(get_contract_balance(id, provider.clone()).await, 1);

        // contract call transfer funds to another address
        // specify amount x, provide amount x - 1
        // fails with panic reason 'NotEnoughBalance'
        let random_wallet = Wallet::random(&mut rand::thread_rng(), provider.clone());
        let (amount, asset_id, recipient) = (
            "5",
            &format!("{{0x{}}}", base_asset_id),
            &format!("(Address:{{0x{}}})", random_wallet.address().hash()),
        );
        let mut cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "transfer",
            vec![amount, asset_id, recipient],
        );
        cmd.call_parameters = cmd::call::CallParametersOpts {
            amount: amount.parse::<u64>().unwrap() - 3,
            asset_id: Some(*base_asset_id),
            gas_forwarded: None,
        };
        cmd.mode = cmd::call::ExecutionMode::Live;
        let operation = cmd.validate_and_get_operation().unwrap();
        assert!(call(operation, cmd)
            .await
            .unwrap_err()
            .to_string()
            .contains("Failed to process transaction; reason: \"NotEnoughBalance\""));
        assert_eq!(get_contract_balance(id, provider.clone()).await, 1);

        // contract call transfer funds to another address
        // specify amount x, provide amount x + 5; should succeed
        let random_wallet = Wallet::random(&mut rand::thread_rng(), provider.clone());
        let (amount, asset_id, recipient) = (
            "3",
            &format!("{{0x{}}}", base_asset_id),
            &format!("(Address:{{0x{}}})", random_wallet.address().hash()),
        );
        let mut cmd = get_contract_call_cmd(
            id,
            node_url,
            secret_key,
            "transfer",
            vec![amount, asset_id, recipient],
        );
        cmd.call_parameters = cmd::call::CallParametersOpts {
            amount: amount.parse::<u64>().unwrap() + 5,
            asset_id: Some(*base_asset_id),
            gas_forwarded: None,
        };
        cmd.mode = cmd::call::ExecutionMode::Live;
        let operation = cmd.validate_and_get_operation().unwrap();
        assert_eq!(call(operation, cmd).await.unwrap().result.unwrap(), "()");
        assert_eq!(
            get_recipient_balance(random_wallet.address().clone(), provider.clone()).await,
            3
        );
        assert_eq!(get_contract_balance(id, provider.clone()).await, 6); // extra amount (5) is forwarded to the contract
    }
}
