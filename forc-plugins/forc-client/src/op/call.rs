use crate::{
    cmd,
    constants::DEFAULT_PRIVATE_KEY,
    util::{
        node_url::{get_explorer_url, get_node_url},
        tx::{prompt_forc_wallet_password, select_local_wallet_account},
    },
};
use anyhow::{anyhow, bail, Result};
use either::Either;
use fuel_abi_types::abi::unified_program::UnifiedProgramABI;
use fuel_tx::{PanicReason, Receipt};
use fuels::{
    crypto::SecretKey,
    programs::calls::{
        receipt_parser::ReceiptParser,
        traits::{ContractDependencyConfigurator, TransactionTuner},
        ContractCall,
    },
};
use fuels_accounts::{provider::Provider, wallet::WalletUnlocked};
use fuels_core::{
    codec::{encode_fn_selector, ABIDecoder, ABIEncoder, DecoderConfig, EncoderConfig},
    types::{
        bech32::Bech32ContractId,
        param_types::ParamType,
        transaction::{Transaction, TxPolicies},
        transaction_builders::{BuildableTransaction, ScriptBuildStrategy, VariableOutputPolicy},
        ContractId, EnumSelector, StaticStringToken, Token, U256,
    },
};
use std::{collections::HashMap, fmt::Write, fs::File, io::Write as _, str::FromStr};
use sway_ast::Item;
use swayfmt::parse::with_handler;

/// A command for calling a contract function.
pub async fn call(cmd: cmd::Call) -> anyhow::Result<String> {
    let cmd::Call {
        contract_id,
        abi,
        function,
        args,
        node,
        caller,
        call_parameters,
        mode,
        gas,
        external_contracts,
    } = cmd;
    let node_url = get_node_url(&node, &None)?;
    let provider: Provider = Provider::connect(node_url.clone()).await?;

    let wallet = get_wallet(caller, provider).await?;

    if let Some(abi) = abi {
        // If ABI is provided, ensure function signature is just the selector
        let cmd::call::FuncType::Selector(selector) = function else {
            bail!("Function must be a selector");
        };

        let (file_path, is_temp_file) = match abi {
            Either::Left(path) => (path, false),
            Either::Right(url) => {
                // Download the file to tempfile; move blocking operations to a new thread
                let path = std::env::temp_dir().join("temp_abi.json");
                let mut file = File::create(&path)?;
                let response = reqwest::get(url).await?.bytes().await?;
                file.write_all(&response)?;
                (path, true)
            }
        };

        let abi_str = std::fs::read_to_string(&file_path).expect("Failed to read ABI file");
        if is_temp_file {
            std::fs::remove_file(file_path).expect("Failed to remove ABI file");
        }
        let parsed_abi = UnifiedProgramABI::from_json_abi(&abi_str)?;

        let type_lookup = parsed_abi
            .types
            .into_iter()
            .map(|decl| (decl.type_id, decl))
            .collect::<HashMap<_, _>>();

        // get the function selector from the abi
        let abi_func = parsed_abi
            .functions
            .iter()
            .find(|abi_func| abi_func.name == selector)
            .unwrap_or_else(|| panic!("Function not found in ABI: {}", selector));

        // println!("function: {:?}", abi_func); // TODO: remove

        if abi_func.inputs.len() != args.len() {
            bail!("Number of arguments does not match number of parameters in function signature");
        }

        let tokens = abi_func
            .inputs
            .iter()
            .zip(&args)
            .map(|(type_application, arg)| {
                // println!("type_application: {:?}", type_application); // TODO: remove

                let param_type =
                    ParamType::try_from_type_application(type_application, &type_lookup)
                        .expect("Failed to convert input type application");
                println!("param_type: {:?}", param_type);
                param_type_val_to_token(&param_type, arg).expect("Failed to convert input value")
            })
            .collect::<Vec<_>>();

        let output_param = ParamType::try_from_type_application(&abi_func.output, &type_lookup)
            .expect("Failed to convert output type");

        let abi_encoder = ABIEncoder::new(EncoderConfig::default());
        let encoded_data = abi_encoder.encode(&tokens)?;

        // Create and execute call
        let call = ContractCall {
            contract_id: contract_id.into(),
            encoded_selector: encode_fn_selector(&selector),
            encoded_args: Ok(encoded_data),
            call_parameters: call_parameters.clone().into(),
            external_contracts: vec![], // set below
            output_param: output_param.clone(),
            is_payable: call_parameters.amount > 0,
            custom_assets: Default::default(),
        };

        let provider = wallet.provider().unwrap();
        // TODO: log decoding would be required for verbose debugging mode
        let log_decoder = fuels::core::codec::LogDecoder::new(
            fuels::core::codec::log_formatters_lookup(vec![], contract_id),
        );

        let tx_policies = gas
            .as_ref()
            .map(Into::into)
            .unwrap_or(TxPolicies::default());
        let variable_output_policy = VariableOutputPolicy::Exactly(call_parameters.amount as usize);
        let external_contracts = match external_contracts {
            Some(external_contracts) => external_contracts
                .iter()
                .map(|addr| Bech32ContractId::from(*addr))
                .collect(),
            None => {
                // Automatically retrieve missing contract addresses from the call - by simulating the call
                // and checking for missing contracts in the receipts
                // This makes the CLI more ergonomic
                let external_contracts = get_missing_contracts(
                    call.clone(),
                    provider,
                    &tx_policies,
                    &variable_output_policy,
                    &log_decoder,
                    &wallet,
                    None,
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

        let (tx_status, tx_hash) = match mode {
            cmd::call::ExecutionMode::DryRun => {
                let tx = call
                    .with_external_contracts(external_contracts)
                    .build_tx(tx_policies, variable_output_policy, &wallet)
                    .await
                    .expect("Failed to build transaction");
                let tx_hash = tx.id(provider.chain_id());
                let tx_status = provider
                    .dry_run(tx)
                    .await
                    .expect("Failed to dry run transaction");
                (tx_status, tx_hash)
            }
            cmd::call::ExecutionMode::Simulate => {
                forc_tracing::println_warning(&format!(
                    "Simulating transaction with wallet... {}",
                    wallet.address().hash()
                ));
                let tx = call
                    .with_external_contracts(external_contracts)
                    .transaction_builder(tx_policies, variable_output_policy, &wallet)
                    .await
                    .expect("Failed to build transaction")
                    .with_build_strategy(ScriptBuildStrategy::StateReadOnly)
                    .build(provider)
                    .await?;
                let tx_hash = tx.id(provider.chain_id());
                let gas_price = gas.map(|g| g.price).unwrap_or(Some(0));
                let tx_status = provider
                    .dry_run_opt(tx, false, gas_price)
                    .await
                    .expect("Failed to simulate transaction");
                (tx_status, tx_hash)
            }
            cmd::call::ExecutionMode::Live => {
                forc_tracing::println_action_green(
                    "Sending transaction with wallet",
                    &format!("0x{}", wallet.address().hash()),
                );
                let tx = call
                    .with_external_contracts(external_contracts)
                    .build_tx(tx_policies, variable_output_policy, &wallet)
                    .await
                    .expect("Failed to build transaction");
                let tx_hash = tx.id(provider.chain_id());
                let tx_status = provider
                    .send_transaction_and_await_commit(tx)
                    .await
                    .expect("Failed to send transaction");
                (tx_status, tx_hash)
            }
        };

        let receipts = tx_status
            .take_receipts_checked(Some(&log_decoder))
            .expect("Failed to take receipts");

        let data = ReceiptParser::new(&receipts, DecoderConfig::default())
            .extract_contract_call_data(contract_id)
            .expect("Failed to extract contract call data");
        let token = ABIDecoder::default()
            .decode(&output_param, &data)
            .expect("Failed to decode output");

        let result = token_to_string(&token).expect("Failed to convert token to string");

        forc_tracing::println_action_green("receipts:", &format!("{:#?}", receipts));
        forc_tracing::println_action_green("tx hash:", &tx_hash.to_string());
        forc_tracing::println_action_green("result:", &result);
        if let Some(explorer_url) = get_explorer_url(&node) {
            forc_tracing::println_action_green(
                "\nView transaction:",
                &format!("{}/tx/0x{}", explorer_url, tx_hash),
            );
        }
        return Ok(result);
    }

    let cmd::call::FuncType::Signature(function_signature) = function else {
        bail!("Function must be a signature if no ABI is provided");
    };

    let parsed = with_handler(|handler| {
        let token_stream = sway_parse::lex(
            handler,
            &function_signature.clone().into(),
            0,
            function_signature.len(),
            None,
        )?;
        sway_parse::Parser::new(handler, &token_stream).parse::<Item>()
    })
    .expect("Parse error");

    eprintln!("{:?}", parsed);

    // function calling via signature unsupported
    bail!("Function calling via signature is unsupported");

    // let handler = Handler::default();
    // let ts = crate::token::lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
    // let r = Parser::new(&handler, &ts).parse();

    // if handler.has_errors() || handler.has_warnings() {
    //     panic!("{:?}", handler.consume());
    // }

    // r.unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
}

async fn get_missing_contracts(
    mut call: ContractCall,
    provider: &Provider,
    tx_policies: &TxPolicies,
    variable_output_policy: &VariableOutputPolicy,
    log_decoder: &fuels_core::codec::LogDecoder,
    account: &WalletUnlocked,
    max_attempts: Option<u64>,
) -> Result<Vec<Bech32ContractId>> {
    let max_attempts = max_attempts.unwrap_or(10);

    for attempt in 1..=max_attempts {
        forc_tracing::println_warning(&format!(
            "Executing dry-run attempt {} to find missing contracts",
            attempt
        ));

        let tx = call
            .build_tx(*tx_policies, *variable_output_policy, account)
            .await?;

        match provider
            .dry_run(tx)
            .await?
            .take_receipts_checked(Some(log_decoder))
        {
            Ok(_) => return Ok(call.external_contracts),
            Err(fuels_core::types::errors::Error::Transaction(
                fuels::types::errors::transaction::Reason::Reverted { receipts, .. },
            )) => match find_id_of_missing_contract(&receipts) {
                Some(contract_id) => call.external_contracts.push(contract_id),
                None => bail!("Failed to find missing contract"),
            },
            Err(err) => bail!(err),
        }
    }
    bail!("Max attempts reached while finding missing contracts")
}

pub fn find_id_of_missing_contract(receipts: &[Receipt]) -> Option<Bech32ContractId> {
    receipts.iter().find_map(|receipt| match receipt {
        Receipt::Panic {
            reason,
            contract_id,
            ..
        } if *reason.reason() == PanicReason::ContractNotInInputs => {
            let contract_id = contract_id
                .expect("panic caused by a contract not in inputs must have a contract id");
            Some(Bech32ContractId::from(contract_id))
        }
        _ => None,
    })
}

async fn get_wallet(caller: cmd::call::Caller, provider: Provider) -> Result<WalletUnlocked> {
    match (caller.signing_key, caller.wallet) {
        (None, false) => {
            let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY).unwrap();
            let wallet = WalletUnlocked::new_from_private_key(secret_key, Some(provider));
            forc_tracing::println_warning(&format!(
                "No signing key or wallet flag provided. Using default signer: 0x{}",
                wallet.address().hash()
            ));
            Ok(wallet)
        }
        (Some(secret_key), false) => {
            let wallet = WalletUnlocked::new_from_private_key(secret_key, Some(provider));
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
            let wallet = WalletUnlocked::new_from_private_key(secret_key, Some(provider));
            Ok(wallet)
        }
    }
}

/// Converts a ParamType and associated value into a Token
pub fn param_type_val_to_token(param_type: &ParamType, input: &str) -> Result<Token> {
    // Parses a string value while preserving quotes and escaped characters
    let parse_string_value = |input: &str| {
        if input.starts_with('"') && input.ends_with('"') {
            // Remove outer quotes and unescape internal quotes
            let without_outer_quotes = &input[1..input.len() - 1];
            without_outer_quotes.replace("\\\"", "\"")
        } else {
            // If no quotes, just trim whitespace
            input.trim().to_string()
        }
    };

    match param_type {
        ParamType::Unit => Ok(Token::Unit),
        ParamType::Bool => bool::from_str(input)
            .map(Token::Bool)
            .map_err(|_| anyhow!("failed to parse bool value: {}", input)),
        ParamType::U8 => u8::from_str(input)
            .map(Token::U8)
            .map_err(|_| anyhow!("failed to parse u8 value: {}", input)),
        ParamType::U16 => u16::from_str(input)
            .map(Token::U16)
            .map_err(|_| anyhow!("failed to parse u16 value: {}", input)),
        ParamType::U32 => u32::from_str(input)
            .map(Token::U32)
            .map_err(|_| anyhow!("failed to parse u32 value: {}", input)),
        ParamType::U64 => u64::from_str(input)
            .map(Token::U64)
            .map_err(|_| anyhow!("failed to parse u64 value: {}", input)),
        ParamType::U128 => u128::from_str(input)
            .map(Token::U128)
            .map_err(|_| anyhow!("failed to parse u128 value: {}", input)),
        ParamType::U256 => {
            // if prefix is 0x, it's a hex string
            if input.starts_with("0x") {
                U256::from_str(input)
                    .map(Token::U256)
                    .map_err(|_| anyhow!("failed to parse U256 value: {}", input))
            } else {
                U256::from_dec_str(input)
                    .map(Token::U256)
                    .map_err(|_| anyhow!("failed to parse U256 value: {}", input))
            }
        }
        ParamType::B256 => {
            // remove 0x prefix if provided
            let input = input.trim_start_matches("0x");
            if input.len() != 64 {
                return Err(anyhow!("B256 value must be 64 hex characters: {}", input));
            }
            hex::decode(input)
                .map(|bytes| Token::B256(bytes.try_into().unwrap()))
                .map_err(|_| anyhow!("failed to parse B256 value: {}", input))
        }
        ParamType::String => Ok(Token::String(parse_string_value(input))),
        ParamType::Bytes => {
            // remove 0x prefix if provided
            let input = input.trim_start_matches("0x");
            if input.len() % 2 != 0 {
                return Err(anyhow!("bytes value must be even length: {}", input));
            }
            hex::decode(input)
                .map(Token::Bytes)
                .map_err(|_| anyhow!("failed to parse bytes value: {}", input))
        }
        ParamType::RawSlice => {
            // remove 0x prefix if provided
            let input = input.trim_start_matches("0x");
            if input.len() % 2 != 0 {
                return Err(anyhow!("raw slice value must be even length: {}", input));
            }
            hex::decode(input)
                .map(Token::RawSlice)
                .map_err(|_| anyhow!("failed to parse raw slice value: {}", input))
        }
        ParamType::StringArray(size) => {
            let parsed_str = parse_string_value(input);
            if parsed_str.len() != *size {
                return Err(anyhow!(
                    "string array length mismatch: expected {}, got {}",
                    size,
                    parsed_str.len()
                ));
            }
            Ok(Token::StringArray(StaticStringToken::new(
                parsed_str,
                Some(*size),
            )))
        }
        ParamType::StringSlice => Ok(Token::StringSlice(StaticStringToken::new(
            parse_string_value(input),
            None,
        ))),
        ParamType::Tuple(types) => {
            // ensure input starts with '(' and ends with ')'
            let parsed_tuple = parse_delimited_string(param_type, input)?;
            Ok(Token::Tuple(
                types
                    .iter()
                    .zip(parsed_tuple.iter())
                    .map(|(ty, s)| param_type_val_to_token(ty, s))
                    .collect::<Result<Vec<_>>>()?,
            ))
        }
        ParamType::Array(ty, _size) => {
            // ensure input starts with '[' and ends with ']'
            let parsed_array = parse_delimited_string(param_type, input)?;
            Ok(Token::Array(
                parsed_array
                    .iter()
                    .map(|s| param_type_val_to_token(ty, s))
                    .collect::<Result<Vec<_>>>()?,
            ))
        }
        ParamType::Vector(ty) => {
            // ensure input starts with '[' and ends with ']'
            let parsed_vector = parse_delimited_string(param_type, input)?;
            Ok(Token::Vector(
                parsed_vector
                    .iter()
                    .map(|s| param_type_val_to_token(ty, s))
                    .collect::<Result<Vec<_>>>()?,
            ))
        }
        ParamType::Struct { fields, .. } => {
            // ensure input starts with '{' and ends with '}'
            let parsed_vals = parse_delimited_string(param_type, input)?;
            let parsed_struct = fields
                .iter()
                .zip(parsed_vals.iter())
                .map(|((_, ty), val)| {
                    println!("ty: {:?}", ty);
                    println!("val: {:?}", val);
                    param_type_val_to_token(ty, val)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Token::Struct(parsed_struct))
        }
        ParamType::Enum { enum_variants, .. } => {
            // enums must start with '(' and end with ')'
            // enums must be in format of (variant_index:variant_value) or (variant_name:variant_value)
            let parsed_enum = parse_delimited_string(param_type, input)?;
            println!("parsed_enum: {:?}", parsed_enum);
            if parsed_enum.len() != 2 {
                bail!(
                    "enum must have exactly two parts `(variant:value)`: {}",
                    input
                );
            }

            let (variant_name_or_index, variant_value) = (&parsed_enum[0], &parsed_enum[1]);
            // if variant can be parsed as u64 it is index; else it is name
            let discriminant = match variant_name_or_index.parse::<u64>() {
                Ok(index) => index,
                Err(_) => {
                    // must be name; find index of variant_name_or_index in enum_variants given
                    let index = enum_variants
                        .variants()
                        .iter()
                        .position(|(name, _)| *name == *variant_name_or_index)
                        .ok_or(anyhow!(
                            "failed to find index of variant: {}",
                            variant_name_or_index
                        ))?;
                    index as u64
                }
            };
            let (_, ty) = enum_variants.select_variant(discriminant).map_err(|_| {
                anyhow!("failed to select enum variant: `{}`", variant_name_or_index)
            })?;
            let token = param_type_val_to_token(ty, variant_value).map_err(|_| {
                anyhow!(
                    "failed to parse `{}` variant enum value: {}",
                    variant_name_or_index,
                    variant_value
                )
            })?;
            let enum_selector: EnumSelector = (discriminant, token, enum_variants.clone());
            Ok(Token::Enum(enum_selector.into()))
        }
    }
}

/// Converts a Token to ParamType - unused unless we want to support input-param validation for enums
#[allow(dead_code)]
pub fn token_to_param_type(token: &Token) -> Result<ParamType> {
    match token {
        Token::Unit => Ok(ParamType::Unit),
        Token::Bool(_) => Ok(ParamType::Bool),
        Token::U8(_) => Ok(ParamType::U8),
        Token::U16(_) => Ok(ParamType::U16),
        Token::U32(_) => Ok(ParamType::U32),
        Token::U64(_) => Ok(ParamType::U64),
        Token::U128(_) => Ok(ParamType::U128),
        Token::U256(_) => Ok(ParamType::U256),
        Token::B256(_) => Ok(ParamType::B256),
        Token::Bytes(_) => Ok(ParamType::Bytes),
        Token::String(_) => Ok(ParamType::String),
        Token::RawSlice(_) => Ok(ParamType::RawSlice),
        Token::StringArray(str) => Ok(ParamType::StringArray(str.get_encodable_str()?.len())),
        Token::StringSlice(_) => Ok(ParamType::StringSlice),
        Token::Tuple(tokens) => Ok(ParamType::Tuple(
            tokens
                .iter()
                .map(token_to_param_type)
                .collect::<Result<Vec<_>>>()?,
        )),
        Token::Array(tokens) => Ok(ParamType::Array(
            Box::new(token_to_param_type(
                &tokens.iter().next().unwrap_or(&Token::default()).clone(),
            )?),
            tokens.len(),
        )),
        Token::Vector(tokens) => Ok(ParamType::Vector(Box::new(token_to_param_type(
            &tokens.iter().next().unwrap_or(&Token::default()).clone(),
        )?))),
        Token::Struct(tokens) => Ok(ParamType::Struct {
            name: "".to_string(),
            fields: tokens
                .iter()
                .map(|t| {
                    (
                        "".to_string(),
                        token_to_param_type(t).expect("failed to convert token to param type"),
                    )
                })
                .collect::<Vec<(String, ParamType)>>(),
            generics: vec![],
        }),
        Token::Enum(boxed_enum) => {
            let (discriminant, _, enum_variants) = &**boxed_enum;
            let (_name, _ty) = enum_variants
                .select_variant(*discriminant)
                .expect("failed to select variant");
            Ok(ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: Default::default(),
            })
        }
    }
}

/// Converts a Token to a string
pub fn token_to_string(token: &Token) -> Result<String> {
    match token {
        Token::Unit => Ok("()".to_string()),
        Token::Bool(b) => Ok(b.to_string()),
        Token::U8(n) => Ok(n.to_string()),
        Token::U16(n) => Ok(n.to_string()),
        Token::U32(n) => Ok(n.to_string()),
        Token::U64(n) => Ok(n.to_string()),
        Token::U128(n) => Ok(n.to_string()),
        Token::U256(n) => Ok(n.to_string()),
        Token::B256(bytes) => {
            let mut hex = String::with_capacity(bytes.len() * 2);
            for byte in bytes {
                write!(hex, "{:02x}", byte).unwrap();
            }
            Ok(format!("0x{}", hex))
        }
        Token::Bytes(bytes) => {
            let mut hex = String::with_capacity(bytes.len() * 2);
            for byte in bytes {
                write!(hex, "{:02x}", byte).unwrap();
            }
            Ok(format!("0x{}", hex))
        }
        Token::String(s) => Ok(s.clone()),
        Token::RawSlice(bytes) => {
            let mut hex = String::with_capacity(bytes.len() * 2);
            for byte in bytes {
                write!(hex, "{:02x}", byte).unwrap();
            }
            Ok(format!("0x{}", hex))
        }
        Token::StringArray(token) => Ok(token.get_encodable_str().map(|s| s.to_string())?),
        Token::StringSlice(token) => token
            .get_encodable_str()
            .map(|s| s.to_string())
            .map_err(|_| anyhow!("failed to get encodable string from StringSlice token")),
        Token::Tuple(tokens) => {
            let inner = tokens
                .iter()
                .map(token_to_string)
                .collect::<Result<Vec<String>>>()?
                .join(", ");
            Ok(format!("({inner})"))
        }
        Token::Array(tokens) => {
            let inner = tokens
                .iter()
                .map(token_to_string)
                .collect::<Result<Vec<String>>>()?
                .join(", ");
            Ok(format!("[{inner}]"))
        }
        Token::Vector(tokens) => {
            let inner = tokens
                .iter()
                .map(token_to_string)
                .collect::<Result<Vec<String>>>()?
                .join(", ");
            Ok(format!("[{inner}]"))
        }
        Token::Struct(tokens) => {
            let inner = tokens
                .iter()
                .map(token_to_string)
                .collect::<Result<Vec<String>>>()?
                .join(", ");
            Ok(format!("{{{inner}}}"))
        }
        Token::Enum(selector) => {
            let (discriminant, value, enum_variants) = &**selector;
            let (name, _ty) = enum_variants
                .select_variant(*discriminant)
                .expect("failed to select variant");
            // TODO: variant validation - currently causing issues since we need deep recursive comparisons..
            // // ensure variant matches expected type
            // let ty_got = token_to_param_type(value).map_err(|_| anyhow!("failed to convert token to param type"))?;
            // if ty_got != *ty {
            //     // ensure all fields match of expected type if struct or enum
            //     match (ty, ty_got.clone()) {
            //         // (ParamType::Struct { fields: ty_fields, .. }, ParamType::Struct { fields: ty_got_fields, .. }) => {
            //         //     for ((_, ty_param), (_, ty_got_param)) in ty_fields.iter().zip(ty_got_fields.iter()) {
            //         //         if ty_param != ty_got_param {
            //         //             return Err(anyhow!("expected type {:?} but got {:?}; mismatch in field: expected {:?}, got {:?}", ty, ty_got, ty_param, ty_got_param));
            //         //         }
            //         //     }
            //         // },
            //         (ParamType::Enum { enum_variants: ty_enum_variants, .. }, ParamType::Enum { enum_variants: ty_got_enum_variants, .. }) => {
            //             for ((_, ty_param), (_, ty_got_param)) in ty_enum_variants.variants().iter().zip(ty_got_enum_variants.variants().iter()) {
            //                 if ty_param != ty_got_param {
            //                     return Err(anyhow!("expected type {:?} but got {:?}; mismatch in variant: expected {:?}, got {:?}", ty, ty_got, ty_param, ty_got_param));
            //                 }
            //             }
            //         },
            //         _ => return Err(anyhow!("expected type {:?} but got {:?}", ty, ty_got)),
            //     }
            // }
            Ok(format!("({}:{})", name, token_to_string(value)?))
        }
    }
}

/// Parses a delimited string into a vector of strings, preserving quoted content and nested structures
fn parse_delimited_string(param_type: &ParamType, input: &str) -> Result<Vec<String>> {
    let input = input.trim();
    let (start_delim, end_delim, separator) = match param_type {
        ParamType::Tuple(_) => ('(', ')', ','),
        ParamType::Array(_, _) | ParamType::Vector(_) => ('[', ']', ','),
        ParamType::Struct { .. } => ('{', '}', ','),
        ParamType::Enum { .. } => ('(', ')', ':'),
        _ => bail!("Unsupported param type: {:?}", param_type),
    };

    if !input.starts_with(start_delim) || !input.ends_with(end_delim) {
        bail!(
            "input must start with '{}' and end with '{}': {}",
            start_delim,
            end_delim,
            input
        );
    }

    let inner = &input[1..input.len() - 1];
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut nesting_level = 0u8;

    for c in inner.chars() {
        match (c, in_quotes, escaped) {
            ('\\', _, false) => {
                escaped = true;
                current.push(c);
            }
            ('"', _, true) => {
                escaped = false;
                current.push(c);
            }
            ('"', false, false) => {
                in_quotes = true;
                current.push(c);
            }
            ('"', true, false) => {
                in_quotes = false;
                current.push(c);
            }
            ('{', false, false) => {
                nesting_level += 1;
                current.push(c);
            }
            ('}', false, false) => {
                nesting_level = nesting_level.saturating_sub(1);
                current.push(c);
            }
            ('(', false, false) => {
                nesting_level += 1;
                current.push(c);
            }
            (')', false, false) => {
                nesting_level = nesting_level.saturating_sub(1);
                current.push(c);
            }
            ('[', false, false) => {
                nesting_level += 1;
                current.push(c);
            }
            (']', false, false) => {
                nesting_level = nesting_level.saturating_sub(1);
                current.push(c);
            }
            (c, false, false) if c == separator && nesting_level == 0 => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                    current = String::new();
                }
            }
            (_, _, _) => {
                escaped = false;
                current.push(c);
            }
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    println!("inner: {}", inner);
    println!("current: {}", current);
    println!("parts: {:?}", parts);

    Ok(parts)
}

// pub (crate) fn ty_to_token(ty: &Ty) -> Result<Token> {
//     match ty {
//         Ty::Path(path) => {
//             let prefix = path.prefix.name.to_string();
//             println!("prefix: {}", prefix);
//             Ok(Token::String(prefix))
//         },
//         _ => bail!("Unsupported type: {:?}", ty),
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use cmd::call::FuncType;
    use fuel_crypto::SecretKey;
    use fuels::prelude::*;
    use fuels_accounts::wallet::{Wallet, WalletUnlocked};
    use fuels_core::types::param_types::EnumVariants;

    abigen!(Contract(
        name = "TestContract",
        abi = "forc-plugins/forc-client/test/data/contract_with_types/contract_with_types-abi.json"
    ));

    async fn get_contract_instance() -> (TestContract<WalletUnlocked>, ContractId, WalletUnlocked) {
        // Launch a local network and deploy the contract
        let mut wallets = launch_custom_provider_and_get_wallets(
            WalletsConfig::new(
                Some(1),             /* Single wallet */
                Some(1),             /* Single coin (UTXO) */
                Some(1_000_000_000), /* Amount per coin */
            ),
            None,
            None,
        )
        .await
        .unwrap();
        let wallet = wallets.pop().unwrap();

        let id = Contract::load_from(
            "../../forc-plugins/forc-client/test/data/contract_with_types/contract_with_types.bin",
            LoadConfiguration::default(),
        )
        .unwrap()
        .deploy(&wallet, TxPolicies::default())
        .await
        .unwrap();

        let instance = TestContract::new(id.clone(), wallet.clone());

        (instance, id.into(), wallet)
    }

    fn get_contract_call_cmd(
        id: ContractId,
        wallet: &WalletUnlocked,
        selector: &str,
        args: &str,
    ) -> cmd::Call {
        // get secret key from wallet - use unsafe because secret_key is private
        // 0000000000000000000000000000000000000000000000000000000000000001
        let secret_key =
            unsafe { std::mem::transmute::<&WalletUnlocked, &(Wallet, SecretKey)>(wallet).1 };
        let vec_args = if args.to_string().is_empty() {
            vec![]
        } else {
            vec![args.to_string()]
        };
        cmd::Call {
            contract_id: id,
            abi: Some(Either::Left(std::path::PathBuf::from(
                "../../forc-plugins/forc-client/test/data/contract_with_types/contract_with_types-abi.json",
            ))),
            function: FuncType::Selector(selector.into()),
            args: vec_args,
            node: crate::NodeTarget {
                node_url: Some(wallet.provider().unwrap().url().to_owned()),
                ..Default::default()
            },
            caller: cmd::call::Caller {
                signing_key: Some(secret_key),
                wallet: false,
            },
            call_parameters: Default::default(),
            mode: cmd::call::ExecutionMode::DryRun,
            gas: None,
            external_contracts: None,
        }
    }

    #[test]
    fn test_parse_delimited_string() {
        // Test with comma separator
        let result = parse_delimited_string(&ParamType::Tuple(vec![]), "(a, b, c)").unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);

        // Test with colon separator
        let result = parse_delimited_string(
            &ParamType::Enum {
                name: "TestEnum".to_string(),
                enum_variants: EnumVariants::new(vec![("".to_string(), ParamType::String)])
                    .unwrap(),
                generics: vec![],
            },
            "(key:value)",
        )
        .unwrap();
        assert_eq!(result, vec!["key", "value"]);

        // Test with spaces around separator
        let result = parse_delimited_string(
            &ParamType::Struct {
                name: "TestStruct".to_string(),
                fields: vec![
                    ("a".to_string(), ParamType::String),
                    ("b".to_string(), ParamType::String),
                    ("c".to_string(), ParamType::String),
                ],
                generics: vec![],
            },
            "{a , b , c}",
        )
        .unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);

        // Test with quoted strings
        let result = parse_delimited_string(
            &ParamType::Vector(Box::new(ParamType::String)),
            "[\"a,b\", c]",
        )
        .unwrap();
        assert_eq!(result, vec!["\"a,b\"", "c"]);

        // Test with escaped quotes
        let result =
            parse_delimited_string(&ParamType::Tuple(vec![]), "(\"\\\"a:b\\\"\", c)").unwrap();
        assert_eq!(result, vec!["\"\\\"a:b\\\"\"", "c"]);

        // Test with separator in quotes
        let result = parse_delimited_string(&ParamType::Tuple(vec![]), "(\"a:b\",c)").unwrap();
        assert_eq!(result, vec!["\"a:b\"", "c"]);
    }

    #[test]
    fn param_type_val_to_token_conversion() {
        // unit
        let token = param_type_val_to_token(&ParamType::Unit, "").unwrap();
        assert_eq!(token, Token::Unit);

        // bool
        let token = param_type_val_to_token(&ParamType::Bool, "true").unwrap();
        assert_eq!(token, Token::Bool(true));

        // u8
        let token = param_type_val_to_token(&ParamType::U8, "42").unwrap();
        assert_eq!(token, Token::U8(42));

        // u16
        let token = param_type_val_to_token(&ParamType::U16, "42").unwrap();
        assert_eq!(token, Token::U16(42));

        // u32
        let token = param_type_val_to_token(&ParamType::U32, "42").unwrap();
        assert_eq!(token, Token::U32(42));

        // u64
        let token = param_type_val_to_token(&ParamType::U64, "42").unwrap();
        assert_eq!(token, Token::U64(42));

        // u128
        let token = param_type_val_to_token(&ParamType::U128, "42").unwrap();
        assert_eq!(token, Token::U128(42));

        // u256 - hex string
        let token = param_type_val_to_token(&ParamType::U256, "0x42").unwrap();
        assert_eq!(token, Token::U256(66.into()));

        // u256 - decimal string
        let token = param_type_val_to_token(&ParamType::U256, "42").unwrap();
        assert_eq!(token, Token::U256(42.into()));

        // u256 - decimal string with leading 0
        let token = param_type_val_to_token(
            &ParamType::U256,
            "0000000000000000000000000000000000000000000000000000000000000042",
        )
        .unwrap();
        assert_eq!(token, Token::U256(42.into()));

        // b256 - hex string, incorrect length
        let token_result = param_type_val_to_token(&ParamType::B256, "0x42");
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "B256 value must be 64 hex characters: 42"
        );

        // b256 - hex string, correct length
        let token = param_type_val_to_token(
            &ParamType::B256,
            "0x0000000000000000000000000000000000000000000000000000000000000042",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::B256([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 66
            ])
        );

        // b256 - no 0x prefix
        let token = param_type_val_to_token(
            &ParamType::B256,
            "0000000000000000000000000000000000000000000000000000000000000042",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::B256([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 66
            ])
        );

        // bytes
        let token = param_type_val_to_token(&ParamType::Bytes, "0x42").unwrap();
        assert_eq!(token, Token::Bytes(vec![66]));

        // bytes - no 0x prefix
        let token = param_type_val_to_token(&ParamType::Bytes, "42").unwrap();
        assert_eq!(token, Token::Bytes(vec![66]));

        // string
        let token = param_type_val_to_token(&ParamType::String, "fuel").unwrap();
        assert_eq!(token, Token::String("fuel".to_string()));

        // raw slice
        let token = param_type_val_to_token(&ParamType::RawSlice, "0x42").unwrap();
        assert_eq!(token, Token::RawSlice(vec![66]));

        // raw slice - no 0x prefix
        let token = param_type_val_to_token(&ParamType::RawSlice, "42").unwrap();
        assert_eq!(token, Token::RawSlice(vec![66]));

        // string array - single val
        let token = param_type_val_to_token(&ParamType::StringArray(4), "fuel").unwrap();
        assert_eq!(
            token,
            Token::StringArray(StaticStringToken::new("fuel".to_string(), Some(4)))
        );

        // string array - incorrect length fails
        let token_result = param_type_val_to_token(&ParamType::StringArray(2), "fuel");
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "string array length mismatch: expected 2, got 4"
        );

        // string slice
        let token = param_type_val_to_token(&ParamType::StringSlice, "fuel").unwrap();
        assert_eq!(
            token,
            Token::StringSlice(StaticStringToken::new("fuel".to_string(), None))
        );

        // tuple - incorrect format
        let token_result = param_type_val_to_token(
            &ParamType::Tuple(vec![ParamType::String, ParamType::String]),
            "fuel, 42",
        );
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "input must start with '(' and end with ')': fuel, 42"
        );

        // tuple
        let token = param_type_val_to_token(
            &ParamType::Tuple(vec![ParamType::String, ParamType::String]),
            "(fuel, 42)",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Tuple(vec![
                Token::String("fuel".to_string()),
                Token::String("42".to_string())
            ])
        );

        // tuple - different param types
        let token = param_type_val_to_token(
            &ParamType::Tuple(vec![ParamType::String, ParamType::U8]),
            "(fuel, 42)",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Tuple(vec![Token::String("fuel".to_string()), Token::U8(42)])
        );

        // array
        let token =
            param_type_val_to_token(&ParamType::Array(ParamType::String.into(), 3), "[fuel, 42]")
                .unwrap();
        assert_eq!(
            token,
            Token::Array(vec![
                Token::String("fuel".to_string()),
                Token::String("42".to_string())
            ])
        );

        // array - incorrect format
        let token_result =
            param_type_val_to_token(&ParamType::Array(ParamType::String.into(), 3), "fuel 42");
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "input must start with '[' and end with ']': fuel 42"
        );

        // vector - correct format
        let token =
            param_type_val_to_token(&ParamType::Vector(ParamType::String.into()), "[fuel, 42]")
                .unwrap();
        assert_eq!(
            token,
            Token::Vector(vec![
                Token::String("fuel".to_string()),
                Token::String("42".to_string())
            ])
        );

        // vector - incorrect format
        let token_result =
            param_type_val_to_token(&ParamType::Vector(ParamType::String.into()), "fuel 42");
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "input must start with '[' and end with ']': fuel 42"
        );

        // struct - correct format; single value
        let token = param_type_val_to_token(
            &ParamType::Struct {
                name: "".to_string(),
                fields: vec![("".to_string(), ParamType::String)],
                generics: vec![],
            },
            "{fuel, 42}",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Struct(vec![Token::String("fuel".to_string())])
        );

        // struct - correct format; multiple values
        let token = param_type_val_to_token(
            &ParamType::Struct {
                name: "".to_string(),
                fields: vec![
                    ("".to_string(), ParamType::String),
                    ("".to_string(), ParamType::String),
                ],
                generics: vec![],
            },
            "{fuel, 42}",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Struct(vec![
                Token::String("fuel".to_string()),
                Token::String("42".to_string())
            ])
        );

        // struct - correct format; multiple values; different param types
        let token = param_type_val_to_token(
            &ParamType::Struct {
                name: "".to_string(),
                fields: vec![
                    ("".to_string(), ParamType::String),
                    ("".to_string(), ParamType::U8),
                ],
                generics: vec![],
            },
            "{fuel, 42}",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Struct(vec![Token::String("fuel".to_string()), Token::U8(42)])
        );

        // struct - incorrect format (same as tuple)
        let token_result = param_type_val_to_token(
            &ParamType::Struct {
                name: "".to_string(),
                fields: vec![("a".to_string(), ParamType::String)],
                generics: vec![],
            },
            "fuel, 42",
        );
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "input must start with '{' and end with '}': fuel, 42"
        );

        // enum - incorrect format
        let token_result = param_type_val_to_token(
            &ParamType::Enum {
                name: "".to_string(),
                enum_variants: EnumVariants::new(vec![
                    ("".to_string(), ParamType::String),
                    ("".to_string(), ParamType::U8),
                ])
                .unwrap(),
                generics: vec![],
            },
            "Active: true",
        );
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "input must start with '(' and end with ')': Active: true"
        );

        // enum - variant not found
        let enum_variants = EnumVariants::new(vec![
            ("".to_string(), ParamType::String),
            ("".to_string(), ParamType::U8),
        ])
        .unwrap();
        let token_result = param_type_val_to_token(
            &ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![],
            },
            "(Active: true)",
        );
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "failed to find index of variant: Active"
        );

        // enum - variant found, incorrect variant value (expect cannot parse u8 as bool)
        let enum_variants = EnumVariants::new(vec![
            ("Input".to_string(), ParamType::String),
            ("Active".to_string(), ParamType::U8),
        ])
        .unwrap();
        let token_result = param_type_val_to_token(
            &ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![],
            },
            "(Active: true)",
        );
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "failed to parse `Active` variant enum value: true"
        );

        // enum - variant found, correct variant value
        let enum_variants = EnumVariants::new(vec![
            ("Input".to_string(), ParamType::String),
            ("Active".to_string(), ParamType::Bool),
        ])
        .unwrap();
        let token = param_type_val_to_token(
            &ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![],
            },
            "(Active: true)",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Enum((1u64, Token::Bool(true), enum_variants).into())
        );

        // enum - variant found by index, incorrect index type (should be bool)
        let enum_variants = EnumVariants::new(vec![
            ("Input".to_string(), ParamType::String),
            ("Active".to_string(), ParamType::Bool),
        ])
        .unwrap();
        let token_result = param_type_val_to_token(
            &ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![],
            },
            "(1: 1)",
        );
        assert!(token_result.is_err());
        assert_eq!(
            token_result.unwrap_err().to_string(),
            "failed to parse `1` variant enum value: 1"
        );

        // enum - variant found by index, correct variant value
        let enum_variants = EnumVariants::new(vec![
            ("Input".to_string(), ParamType::String),
            ("Active".to_string(), ParamType::Bool),
        ])
        .unwrap();
        let token = param_type_val_to_token(
            &ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![],
            },
            "(1: true)",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Enum((1u64, Token::Bool(true), enum_variants).into())
        );

        // enum (complex example) - variants with a struct that contains an enum and a vec that contains another enum with 2 variants
        let enum_variants = EnumVariants::new(vec![
            (
                "Input".to_string(),
                ParamType::Struct {
                    generics: vec![],
                    name: "".to_string(),
                    fields: vec![
                        (
                            "".to_string(),
                            ParamType::Enum {
                                name: "".to_string(),
                                enum_variants: EnumVariants::new(vec![
                                    ("Active".to_string(), ParamType::Bool),
                                    ("Pending".to_string(), ParamType::U64),
                                ])
                                .unwrap(),
                                generics: vec![],
                            },
                        ),
                        (
                            "".to_string(),
                            ParamType::Vector(Box::new(ParamType::Enum {
                                name: "".to_string(),
                                enum_variants: EnumVariants::new(vec![
                                    ("Active".to_string(), ParamType::Bool),
                                    ("Pending".to_string(), ParamType::U64),
                                ])
                                .unwrap(),
                                generics: vec![],
                            })),
                        ),
                    ],
                },
            ),
            ("Active".to_string(), ParamType::Bool),
        ])
        .unwrap();
        let token = param_type_val_to_token(
            &ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![],
            },
            "(Input: {(Active: true), [(Pending: 42)]})",
        )
        .unwrap();
        assert_eq!(
            token,
            Token::Enum(
                (
                    0u64,
                    Token::Struct(vec![
                        Token::Enum(
                            (
                                0u64,
                                Token::Bool(true),
                                EnumVariants::new(vec![
                                    ("Active".to_string(), ParamType::Bool),
                                    ("Pending".to_string(), ParamType::U64)
                                ])
                                .unwrap()
                            )
                                .into()
                        ),
                        Token::Vector(vec![Token::Enum(
                            (
                                1u64,
                                Token::U64(42),
                                EnumVariants::new(vec![
                                    ("Active".to_string(), ParamType::Bool),
                                    ("Pending".to_string(), ParamType::U64)
                                ])
                                .unwrap()
                            )
                                .into()
                        )])
                    ]),
                    enum_variants
                )
                    .into()
            )
        );
    }

    #[test]
    fn token_to_param_type_conversion() {
        // unit
        let token = Token::Unit;
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::Unit);

        // bool
        let token = Token::Bool(true);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::Bool);

        // u8
        let token = Token::U8(42);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::U8);

        // u16
        let token = Token::U16(42);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::U16);

        // u32
        let token = Token::U32(42);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::U32);

        // u64
        let token = Token::U64(42);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::U64);

        // u128
        let token = Token::U128(42);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::U128);

        // u256
        let token = Token::U256(42.into());
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::U256);

        // b256
        let token = Token::B256([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 66,
        ]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::B256);

        // bytes
        let token = Token::Bytes(vec![66]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::Bytes);

        // string
        let token = Token::String("fuel".to_string());
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::String);

        // raw slice
        let token = Token::RawSlice(vec![66]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::RawSlice);

        // string array
        let token = Token::StringArray(StaticStringToken::new("fuel".to_string(), Some(4)));
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::StringArray(4));

        // string slice
        let token = Token::StringSlice(StaticStringToken::new("fuel".to_string(), None));
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::StringSlice);

        // tuple
        let token = Token::Tuple(vec![Token::String("fuel".to_string()), Token::U8(42)]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(
            param_type,
            ParamType::Tuple(vec![ParamType::String, ParamType::U8])
        );

        // array
        let token = Token::Array(vec![
            Token::String("fuel".to_string()),
            Token::String("rocks".to_string()),
        ]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::Array(Box::new(ParamType::String), 2));

        // vector
        let token = Token::Vector(vec![
            Token::String("fuel".to_string()),
            Token::String("rocks".to_string()),
        ]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(param_type, ParamType::Vector(Box::new(ParamType::String)));

        // struct
        let token = Token::Struct(vec![Token::String("fuel".to_string()), Token::U8(42)]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(
            param_type,
            ParamType::Struct {
                name: "".to_string(),
                fields: vec![
                    ("".to_string(), ParamType::String),
                    ("".to_string(), ParamType::U8)
                ],
                generics: vec![]
            }
        );

        // struct (complex example) - struct with 2 fields that contains another struct with 2 fields
        let token = Token::Struct(vec![
            Token::Struct(vec![Token::U32(42), Token::U32(42)]),
            Token::U32(42),
        ]);
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(
            param_type,
            ParamType::Struct {
                name: "".to_string(),
                fields: vec![
                    (
                        "".to_string(),
                        ParamType::Struct {
                            name: "".to_string(),
                            fields: vec![
                                ("".to_string(), ParamType::U32),
                                ("".to_string(), ParamType::U32)
                            ],
                            generics: vec![]
                        }
                    ),
                    ("".to_string(), ParamType::U32)
                ],
                generics: vec![]
            }
        );

        // enum
        let token = Token::Enum(
            (
                0u64,
                Token::U32(42),
                EnumVariants::new(vec![
                    ("Active".to_string(), ParamType::Bool),
                    ("Pending".to_string(), ParamType::U64),
                ])
                .unwrap(),
            )
                .into(),
        );
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(
            param_type,
            ParamType::Enum {
                name: "".to_string(),
                enum_variants: EnumVariants::new(vec![
                    ("Active".to_string(), ParamType::Bool),
                    ("Pending".to_string(), ParamType::U64)
                ])
                .unwrap(),
                generics: vec![]
            }
        );

        // enum (complex example) - variants with a struct that contains an enum and a vec that contains another enum with 2 variants
        let inner_enum_variants = EnumVariants::new(vec![
            ("Active".to_string(), ParamType::Bool),
            ("Pending".to_string(), ParamType::U64),
        ])
        .unwrap();
        let enum_variants = EnumVariants::new(vec![
            (
                "Input".to_string(),
                ParamType::Struct {
                    generics: vec![],
                    name: "".to_string(),
                    fields: vec![
                        (
                            "".to_string(),
                            ParamType::Enum {
                                name: "".to_string(),
                                enum_variants: inner_enum_variants.clone(),
                                generics: vec![],
                            },
                        ),
                        (
                            "".to_string(),
                            ParamType::Vector(Box::new(ParamType::Enum {
                                name: "".to_string(),
                                enum_variants: inner_enum_variants.clone(),
                                generics: vec![],
                            })),
                        ),
                    ],
                },
            ),
            ("Active".to_string(), ParamType::Bool),
        ])
        .unwrap();
        let token = Token::Enum(
            (
                0u64,
                Token::Struct(vec![
                    Token::Enum((0u64, Token::Bool(true), inner_enum_variants.clone()).into()),
                    Token::Vector(vec![Token::Enum(
                        (1u64, Token::U64(42), inner_enum_variants.clone()).into(),
                    )]),
                ]),
                enum_variants.clone(),
            )
                .into(),
        );
        let param_type = token_to_param_type(&token).unwrap();
        assert_eq!(
            param_type,
            ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![]
            }
        );

        // enum (complex example 2) - 2 variants, one with a struct that contains another struct with 2 fields, another with a struct
        // enum GenericEnum<GenericStruct> {
        //     container: GenericStruct<u32> {
        //         value: GenericStruct {
        //             value: u32,
        //             description: str[4],
        //         },
        //         description: str[4],
        //     },
        //     value: GenericStruct<u32> {
        //         value: u32,
        //         description: str[4],
        //     },
        // }
        let inner_struct = ParamType::Struct {
            generics: vec![ParamType::U32],
            name: "GenericStruct".to_string(),
            fields: vec![
                ("value".to_string(), ParamType::U32),
                ("description".to_string(), ParamType::StringArray(4)),
            ],
        };
        let enum_variants = EnumVariants::new(vec![
            (
                "container".to_string(),
                ParamType::Struct {
                    name: "GenericStruct".to_string(),
                    generics: vec![ParamType::Struct {
                        name: "GenericStruct".to_string(),
                        fields: vec![
                            ("value".to_string(), ParamType::U32),
                            ("description".to_string(), ParamType::StringArray(4)),
                        ],
                        generics: vec![ParamType::U32],
                    }],
                    fields: vec![
                        ("value".to_string(), inner_struct.clone()),
                        ("description".to_string(), ParamType::StringArray(4)),
                    ],
                },
            ),
            ("value".to_string(), inner_struct.clone()),
        ])
        .unwrap();
        let token = Token::Enum(
            (
                0u64,
                Token::Struct(vec![
                    Token::Struct(vec![
                        Token::U32(42),
                        Token::StringArray(StaticStringToken::new("fuel".into(), Some(4))),
                    ]),
                    Token::StringArray(StaticStringToken::new("fuel".into(), Some(4))),
                ]),
                enum_variants.clone(),
            )
                .into(),
        );
        let output = token_to_param_type(&token).unwrap();
        assert_eq!(
            output,
            ParamType::Enum {
                name: "".to_string(),
                enum_variants: enum_variants.clone(),
                generics: vec![]
            }
        );
    }

    #[test]
    fn token_to_string_conversion() {
        // unit
        let token = Token::Unit;
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "()");

        // bool
        let token = Token::Bool(true);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "true");

        // u8
        let token = Token::U8(42);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "42");

        // u16
        let token = Token::U16(42);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "42");

        // u32
        let token = Token::U32(42);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "42");

        // u64
        let token = Token::U64(42);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "42");

        // u128
        let token = Token::U128(42);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "42");

        // u256
        let token = Token::U256(42.into());
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "42");

        // b256
        let token = Token::B256([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 66,
        ]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(
            output,
            "0x0000000000000000000000000000000000000000000000000000000000000042"
        );

        // bytes
        let token = Token::Bytes(vec![66]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "0x42");

        // string
        let token = Token::String("fuel".to_string());
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "fuel");

        // raw slice
        let token = Token::RawSlice(vec![66]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "0x42");

        // string array - fails if length is incorrect
        let token = Token::StringArray(StaticStringToken::new("fuel".to_string(), Some(1)));
        let output_res = token_to_string(&token);
        assert!(output_res.is_err());
        assert_eq!(
            output_res.unwrap_err().to_string(),
            "codec: string data has len 4, but the expected len is 1"
        );

        // string array - fails if length overflows
        let token = Token::StringArray(StaticStringToken::new("fuel".to_string(), Some(10)));
        let output_res = token_to_string(&token);
        assert!(output_res.is_err());
        assert_eq!(
            output_res.unwrap_err().to_string(),
            "codec: string data has len 4, but the expected len is 10"
        );

        // string array - succeeds if length not provided
        // TODO: probably an issue in the SDK; should fail validation
        let token = Token::StringArray(StaticStringToken::new("fuel".to_string(), None));
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "fuel");

        // string array - succeeds if length is correct
        let token = Token::StringArray(StaticStringToken::new("fuel".to_string(), Some(4)));
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "fuel");

        // string slice
        let token = Token::StringSlice(StaticStringToken::new("fuel".to_string(), None));
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "fuel");

        // tuple
        let token = Token::Tuple(vec![Token::String("fuel".to_string()), Token::U8(42)]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "(fuel, 42)");

        // array - same param types
        let token = Token::Array(vec![
            Token::String("fuel".to_string()),
            Token::String("rocks".to_string()),
        ]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "[fuel, rocks]");

        // array - different param types
        // TODO: probably an issue in the SDK; should fail validation
        let token = Token::Array(vec![Token::String("fuel".to_string()), Token::U8(42)]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "[fuel, 42]");

        // vector - same param types
        let token = Token::Vector(vec![
            Token::String("fuel".to_string()),
            Token::String("rocks".to_string()),
        ]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "[fuel, rocks]");

        // vector - different param types
        // TODO: probably an issue in the SDK; should fail validation
        let token = Token::Vector(vec![Token::String("fuel".to_string()), Token::U8(42)]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "[fuel, 42]");

        // struct - single value
        let token = Token::Struct(vec![Token::String("fuel".to_string())]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "{fuel}");

        // struct - multiple values
        let token = Token::Struct(vec![Token::String("fuel".to_string()), Token::U8(42)]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "{fuel, 42}");

        // struct (complex example) - struct with 2 fields that contains another struct with 2 fields
        let token = Token::Struct(vec![
            Token::Struct(vec![Token::U32(42), Token::U32(42)]),
            Token::U32(42),
        ]);
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "{{42, 42}, 42}");

        // TODO: potentially re-enable this if we want to support input-param validation
        // // enum - fails if variant incorrect
        // let enum_variants = EnumVariants::new(vec![("Active".to_string(), ParamType::Bool), ("Pending".to_string(), ParamType::U64)]).unwrap();
        // let token = Token::Enum((1u64, Token::Bool(true), enum_variants).into());
        // let output_res = token_to_string(&token);
        // assert!(output_res.is_err());
        // assert_eq!(output_res.unwrap_err().to_string(), "expected type U64 but got Bool");

        // enum - correct variant
        let enum_variants = EnumVariants::new(vec![
            ("Active".to_string(), ParamType::Bool),
            ("Pending".to_string(), ParamType::U64),
        ])
        .unwrap();
        let token = Token::Enum((1u64, Token::U64(42), enum_variants).into());
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "(Pending:42)");

        // enum (complex example) - variants with a struct that contains an enum and a vec that contains another enum with 2 variants
        let inner_enum_variants = EnumVariants::new(vec![
            ("Active".to_string(), ParamType::Bool),
            ("Pending".to_string(), ParamType::U64),
        ])
        .unwrap();
        let enum_variants = EnumVariants::new(vec![
            (
                "Input".to_string(),
                ParamType::Struct {
                    generics: vec![],
                    name: "".to_string(),
                    fields: vec![
                        (
                            "".to_string(),
                            ParamType::Enum {
                                name: "".to_string(),
                                enum_variants: inner_enum_variants.clone(),
                                generics: vec![],
                            },
                        ),
                        (
                            "".to_string(),
                            ParamType::Vector(Box::new(ParamType::Enum {
                                name: "".to_string(),
                                enum_variants: inner_enum_variants.clone(),
                                generics: vec![],
                            })),
                        ),
                    ],
                },
            ),
            ("Active".to_string(), ParamType::Bool),
            ("Pending".to_string(), ParamType::U64),
        ])
        .unwrap();

        // test active variant
        let token = Token::Enum((1u64, Token::Bool(true), enum_variants.clone()).into());
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "(Active:true)");

        // test Input variant
        let token = Token::Enum(
            (
                0u64,
                Token::Struct(vec![
                    Token::Enum((0u64, Token::Bool(true), inner_enum_variants.clone()).into()),
                    Token::Vector(vec![Token::Enum(
                        (1u64, Token::U64(42), inner_enum_variants.clone()).into(),
                    )]),
                ]),
                enum_variants,
            )
                .into(),
        );
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "(Input:{(Active:true), [(Pending:42)]})");

        // enum (complex example 2) - 2 variants, one with a struct that contains another struct with 2 fields, another with a struct
        // enum GenericEnum<GenericStruct<u32>> {
        //     container: GenericStruct<u32> {
        //         value: GenericStruct<u32> {
        //             value: u32,
        //             description: str[4],
        //         },
        //         description: str[4],
        //     },
        //     value: GenericStruct<u32> {
        //         value: u32,
        //         description: str[4],
        //     },
        // }
        let inner_struct = ParamType::Struct {
            generics: vec![ParamType::U32],
            name: "GenericStruct".to_string(),
            fields: vec![
                ("value".to_string(), ParamType::U32),
                ("description".to_string(), ParamType::StringArray(4)),
            ],
        };
        let enum_variants = EnumVariants::new(vec![
            (
                "container".to_string(),
                ParamType::Struct {
                    generics: vec![ParamType::Struct {
                        name: "GenericStruct".to_string(),
                        fields: vec![
                            ("value".to_string(), ParamType::U32),
                            ("description".to_string(), ParamType::StringArray(4)),
                        ],
                        generics: vec![ParamType::U32],
                    }],
                    name: "GenericStruct".to_string(),
                    fields: vec![
                        ("value".to_string(), inner_struct.clone()),
                        ("description".to_string(), ParamType::StringArray(4)),
                    ],
                },
            ),
            ("value".to_string(), inner_struct.clone()),
        ])
        .unwrap();
        let token = Token::Enum(
            (
                0u64,
                Token::Struct(vec![
                    Token::Struct(vec![
                        Token::U32(42),
                        Token::StringArray(StaticStringToken::new("fuel".into(), Some(4))),
                    ]),
                    Token::StringArray(StaticStringToken::new("fuel".into(), Some(4))),
                ]),
                enum_variants,
            )
                .into(),
        );
        let output = token_to_string(&token).unwrap();
        assert_eq!(output, "(container:{{42, fuel}, fuel})");
    }

    #[tokio::test]
    async fn contract_call_with_abi() {
        let (_, id, wallet) = get_contract_instance().await;

        // test_empty_no_return
        let cmd = get_contract_call_cmd(id, &wallet, "test_empty_no_return", "");
        assert_eq!(call(cmd).await.unwrap(), "()");

        // test_empty
        let cmd = get_contract_call_cmd(id, &wallet, "test_empty", "");
        assert_eq!(call(cmd).await.unwrap(), "()");

        // test_unit
        let cmd = get_contract_call_cmd(id, &wallet, "test_unit", "()");
        assert_eq!(call(cmd).await.unwrap(), "()");

        // test_u8
        let cmd = get_contract_call_cmd(id, &wallet, "test_u8", "255");
        assert_eq!(call(cmd).await.unwrap(), "255");

        // test_u16
        let cmd = get_contract_call_cmd(id, &wallet, "test_u16", "65535");
        assert_eq!(call(cmd).await.unwrap(), "65535");

        // test_u32
        let cmd = get_contract_call_cmd(id, &wallet, "test_u32", "4294967295");
        assert_eq!(call(cmd).await.unwrap(), "4294967295");

        // test_u64
        let cmd = get_contract_call_cmd(id, &wallet, "test_u64", "18446744073709551615");
        assert_eq!(call(cmd).await.unwrap(), "18446744073709551615");

        // test_u128
        let cmd = get_contract_call_cmd(
            id,
            &wallet,
            "test_u128",
            "340282366920938463463374607431768211455",
        );
        assert_eq!(
            call(cmd).await.unwrap(),
            "340282366920938463463374607431768211455"
        );

        // test_u256
        let cmd = get_contract_call_cmd(
            id,
            &wallet,
            "test_u256",
            "115792089237316195423570985008687907853269984665640564039457584007913129639935",
        );
        assert_eq!(
            call(cmd).await.unwrap(),
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        );

        // test b256
        let cmd = get_contract_call_cmd(
            id,
            &wallet,
            "test_b256",
            "0000000000000000000000000000000000000000000000000000000000000042",
        );
        assert_eq!(
            call(cmd).await.unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000000042"
        );

        // test_b256 - fails if 0x prefix provided since it extracts input as an external contract; we don't want to do this so explicitly provide the external contract as empty
        let mut cmd = get_contract_call_cmd(
            id,
            &wallet,
            "test_b256",
            "0x0000000000000000000000000000000000000000000000000000000000000042",
        );
        cmd.external_contracts = Some(vec![]);
        assert_eq!(
            call(cmd).await.unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000000042"
        );

        // test_bytes
        let cmd = get_contract_call_cmd(id, &wallet, "test_bytes", "0x42");
        assert_eq!(call(cmd).await.unwrap(), "0x42");

        // test bytes without 0x prefix
        let cmd = get_contract_call_cmd(id, &wallet, "test_bytes", "42");
        assert_eq!(call(cmd).await.unwrap(), "0x42");

        // test_str
        let cmd = get_contract_call_cmd(id, &wallet, "test_str", "fuel");
        assert_eq!(call(cmd).await.unwrap(), "fuel");

        // test str array
        let cmd = get_contract_call_cmd(id, &wallet, "test_str_array", "fuel rocks");
        assert_eq!(call(cmd).await.unwrap(), "fuel rocks");

        // test str array - fails if length mismatch
        let cmd = get_contract_call_cmd(id, &wallet, "test_str_array", "fuel");
        assert_eq!(
            call(cmd).await.unwrap_err().to_string(),
            "string array length mismatch: expected 10, got 4"
        );

        // test str slice
        let cmd = get_contract_call_cmd(id, &wallet, "test_str_slice", "fuel rocks 42");
        assert_eq!(call(cmd).await.unwrap(), "fuel rocks 42");

        // test tuple
        let cmd = get_contract_call_cmd(id, &wallet, "test_tuple", "(42, true)");
        assert_eq!(call(cmd).await.unwrap(), "(42, true)");

        // test array
        let cmd = get_contract_call_cmd(
            id,
            &wallet,
            "test_array",
            "[42, 42, 42, 42, 42, 42, 42, 42, 42, 42]",
        );
        assert_eq!(
            call(cmd).await.unwrap(),
            "[42, 42, 42, 42, 42, 42, 42, 42, 42, 42]"
        );

        // test_array - fails if different types
        let cmd = get_contract_call_cmd(id, &wallet, "test_array", "[42, true]");
        assert_eq!(
            call(cmd).await.unwrap_err().to_string(),
            "failed to parse u64 value: true"
        );

        // test_array - succeeds if length not matched!?
        let cmd = get_contract_call_cmd(id, &wallet, "test_array", "[42, 42]");
        assert_eq!(
            call(cmd).await.unwrap(),
            "[42, 42, 0, 4718592, 65536, 65536, 0, 0, 0, 0]"
        );

        // test_vector
        let cmd = get_contract_call_cmd(id, &wallet, "test_vector", "[42, 42]");
        assert_eq!(call(cmd).await.unwrap(), "[42, 42]");

        // test_vector - fails if different types
        let cmd = get_contract_call_cmd(id, &wallet, "test_vector", "[42, true]");
        assert_eq!(
            call(cmd).await.unwrap_err().to_string(),
            "failed to parse u64 value: true"
        );

        // test_struct - Identity { name: str[2], id: u64 }
        let cmd = get_contract_call_cmd(id, &wallet, "test_struct", "{fu, 42}");
        assert_eq!(call(cmd).await.unwrap(), "{fu, 42}");

        // test_struct - fails if incorrect inner attribute length
        let cmd = get_contract_call_cmd(id, &wallet, "test_struct", "{fuel, 42}");
        assert_eq!(
            call(cmd).await.unwrap_err().to_string(),
            "string array length mismatch: expected 2, got 4"
        );

        // test_struct - succeeds if missing inner final attribute; default value is used
        let cmd = get_contract_call_cmd(id, &wallet, "test_struct", "{fu}");
        assert_eq!(call(cmd).await.unwrap(), "{fu, 0}");

        // test_struct - succeeds to use default values for all attributes if missing
        let cmd = get_contract_call_cmd(id, &wallet, "test_struct", "{}");
        assert_eq!(call(cmd).await.unwrap(), "{\0\0, 0}");

        // test_enum
        let cmd = get_contract_call_cmd(id, &wallet, "test_enum", "(Active:true)");
        assert_eq!(call(cmd).await.unwrap(), "(Active:true)");

        // test_enum - succeeds if using index
        let cmd = get_contract_call_cmd(id, &wallet, "test_enum", "(1:56)");
        assert_eq!(call(cmd).await.unwrap(), "(Pending:56)");

        // test_enum - fails if variant not found
        let cmd = get_contract_call_cmd(id, &wallet, "test_enum", "(A:true)");
        assert_eq!(
            call(cmd).await.unwrap_err().to_string(),
            "failed to find index of variant: A"
        );

        // test_enum - fails if variant value incorrect
        let cmd = get_contract_call_cmd(id, &wallet, "test_enum", "(Active:3)");
        assert_eq!(
            call(cmd).await.unwrap_err().to_string(),
            "failed to parse `Active` variant enum value: 3"
        );

        // test_enum - fails if variant value is missing
        let cmd = get_contract_call_cmd(id, &wallet, "test_enum", "(Active:)");
        assert_eq!(
            call(cmd).await.unwrap_err().to_string(),
            "enum must have exactly two parts `(variant:value)`: (Active:)"
        );

        // test_option - encoded like an enum
        let cmd = get_contract_call_cmd(id, &wallet, "test_option", "(0:())");
        assert_eq!(call(cmd).await.unwrap(), "(None:())");

        // test_option - encoded like an enum; none value ignored
        let cmd = get_contract_call_cmd(id, &wallet, "test_option", "(0:42)");
        assert_eq!(call(cmd).await.unwrap(), "(None:())");

        // test_option - encoded like an enum; some value
        let cmd = get_contract_call_cmd(id, &wallet, "test_option", "(1:42)");
        assert_eq!(call(cmd).await.unwrap(), "(Some:42)");
    }

    #[tokio::test]
    async fn contract_call_with_abi_complex() {
        let (_, id, wallet) = get_contract_instance().await;

        // test_complex_struct
        let cmd = get_contract_call_cmd(id, &wallet, "test_struct_with_generic", "{42, fuel}");
        assert_eq!(call(cmd).await.unwrap(), "{42, fuel}");

        // test_enum_with_generic
        let cmd = get_contract_call_cmd(id, &wallet, "test_enum_with_generic", "(value:32)");
        assert_eq!(call(cmd).await.unwrap(), "(value:32)");

        // test_enum_with_complex_generic
        let cmd = get_contract_call_cmd(
            id,
            &wallet,
            "test_enum_with_complex_generic",
            "(value:{42, fuel})",
        );
        assert_eq!(call(cmd).await.unwrap(), "(value:{42, fuel})");

        let cmd = get_contract_call_cmd(
            id,
            &wallet,
            "test_enum_with_complex_generic",
            "(container:{{42, fuel}, fuel})",
        );
        assert_eq!(call(cmd).await.unwrap(), "(container:{{42, fuel}, fuel})");
    }
}
