use crate::{
    constants::DEFAULT_PRIVATE_KEY,
    util::{account::ForcClientAccount, aws::AwsSigner, target::Target},
};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Password, Select};
use forc_tracing::{println_action_green, println_warning};
use forc_wallet::{
    account::{derive_secret_key, new_at_index_cli},
    balance::{collect_accounts_with_verification, AccountBalances, AccountVerification},
    import::{import_wallet_cli, Import},
    new::{new_wallet_cli, New},
    utils::default_wallet_path,
};
use fuel_crypto::SecretKey;
use fuel_tx::{AssetId, ContractId};
use fuels::{
    macros::abigen, programs::responses::CallResponse, types::checksum_address::checksum_encode,
};
use fuels_accounts::{
    provider::Provider,
    signers::private_key::PrivateKeySigner,
    wallet::{Unlocked, Wallet},
    ViewOnlyAccount,
};

use std::{collections::BTreeMap, path::Path, str::FromStr};

use super::aws::{AwsClient, AwsConfig};

type AccountsMap = BTreeMap<usize, fuel_tx::Address>;

#[derive(PartialEq, Eq)]
pub enum SignerSelectionMode {
    /// Holds the password of forc-wallet instance.
    ForcWallet(String),
    /// Holds ARN of the AWS signer.
    AwsSigner(String),
    Manual,
}

fn ask_user_yes_no_question(question: &str) -> Result<bool> {
    let answer = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(false)
        .show_default(false)
        .interact()?;
    Ok(answer)
}

fn ask_user_with_options(question: &str, options: &[&str], default: usize) -> Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .items(options)
        .default(default)
        .interact()?;
    Ok(selection)
}

async fn collect_user_accounts(
    wallet_path: &Path,
    password: &str,
    node_url: &str,
) -> Result<AccountsMap> {
    let verification = AccountVerification::Yes(password.to_string());
    let node_url = reqwest::Url::parse(node_url)
        .map_err(|e| anyhow::anyhow!("Failed to parse node URL: {}", e))?;
    let accounts = collect_accounts_with_verification(wallet_path, verification, &node_url)
        .await
        .map_err(|e| {
            if e.to_string().contains("Mac Mismatch") {
                anyhow::anyhow!("Failed to access forc-wallet vault. Please check your password")
            } else {
                e
            }
        })?;
    let accounts = accounts
        .into_iter()
        .map(|(index, address)| {
            let bytes: [u8; fuel_tx::Address::LEN] = address.into();
            (index, fuel_tx::Address::from(bytes))
        })
        .collect();
    Ok(accounts)
}

pub(crate) fn prompt_forc_wallet_password() -> Result<String> {
    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Wallet password")
        .allow_empty_password(true)
        .interact()?;

    Ok(password)
}

pub(crate) async fn check_and_create_wallet_at_default_path(wallet_path: &Path) -> Result<()> {
    if !wallet_path.exists() {
        let question =
            format!("Could not find a wallet at {wallet_path:?}, please select an option: ");
        let wallet_options = ask_user_with_options(
            &question,
            &["Create new wallet", "Import existing wallet"],
            0,
        )?;
        let ctx = forc_wallet::CliContext {
            wallet_path: wallet_path.to_path_buf(),
            node_url: forc_wallet::network::DEFAULT.parse().unwrap(),
        };
        match wallet_options {
            0 => {
                new_wallet_cli(&ctx, New { force: false, cache_accounts: None }).await?;
                println!("Wallet created successfully.");
            }
            1 => {
                import_wallet_cli(&ctx, Import { force: false, cache_accounts: None }).await?;
                println!("Wallet imported successfully.");
            },
            _ => anyhow::bail!("Refused to create or import a new wallet. If you don't want to use forc-wallet, you can sign this transaction manually with --manual-signing flag."),
        }
        // Derive first account for the fresh wallet we created.
        new_at_index_cli(&ctx, 0).await?;
        println!("Account derived successfully.");
    }
    Ok(())
}

pub(crate) fn secret_key_from_forc_wallet(
    wallet_path: &Path,
    account_index: usize,
    password: &str,
) -> Result<SecretKey> {
    let secret_key = derive_secret_key(wallet_path, account_index, password).map_err(|e| {
        if e.to_string().contains("Mac Mismatch") {
            anyhow::anyhow!("Failed to access forc-wallet vault. Please check your password")
        } else {
            e
        }
    })?;
    SecretKey::try_from(secret_key.as_ref())
        .map_err(|e| anyhow::anyhow!("Failed to convert secret key: {e}"))
}

pub(crate) fn select_manual_secret_key(
    default_signer: bool,
    signing_key: Option<SecretKey>,
) -> Option<SecretKey> {
    match (default_signer, signing_key) {
        // Note: unwrap is safe here as we already know that 'DEFAULT_PRIVATE_KEY' is a valid private key.
        (true, None) => Some(SecretKey::from_str(DEFAULT_PRIVATE_KEY).unwrap()),
        (true, Some(signing_key)) => {
            println_warning("Signing key is provided while requesting to sign with a default signer. Using signing key");
            Some(signing_key)
        }
        (false, None) => None,
        (false, Some(signing_key)) => Some(signing_key),
    }
}

/// Collect and return balances of each account in the accounts map.
async fn collect_account_balances(
    accounts_map: &AccountsMap,
    provider: &Provider,
) -> Result<AccountBalances> {
    let accounts: Vec<_> = accounts_map
        .values()
        .map(|addr| Wallet::new_locked(*addr, provider.clone()))
        .collect();

    futures::future::try_join_all(accounts.iter().map(|acc| acc.get_balances()))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Format collected account balances for each asset type, including only the balance of the base asset that can be used to pay gas.
pub fn format_base_asset_account_balances(
    accounts_map: &AccountsMap,
    account_balances: &AccountBalances,
    base_asset_id: &AssetId,
) -> Result<Vec<String>> {
    accounts_map
        .iter()
        .zip(account_balances)
        .map(|((ix, address), balance)| {
            let base_asset_amount = balance
                .get(&base_asset_id.to_string())
                .copied()
                .unwrap_or(0);
            let raw_addr = format!("0x{address}");
            let checksum_addr = checksum_encode(&raw_addr)?;
            let eth_amount = base_asset_amount as f64 / 1_000_000_000.0;
            Ok(format!("[{ix}] {checksum_addr} - {eth_amount} ETH"))
        })
        .collect::<Result<Vec<_>>>()
}

// TODO: Simplify the function signature once https://github.com/FuelLabs/sway/issues/6071 is closed.
pub(crate) async fn select_account(
    wallet_mode: &SignerSelectionMode,
    default_sign: bool,
    signing_key: Option<SecretKey>,
    provider: &Provider,
    tx_count: usize,
) -> Result<ForcClientAccount> {
    let chain_info = provider.chain_info().await?;
    match wallet_mode {
        SignerSelectionMode::ForcWallet(password) => {
            let wallet_path = default_wallet_path();
            let accounts = collect_user_accounts(&wallet_path, password, provider.url()).await?;
            let account_balances = collect_account_balances(&accounts, provider).await?;

            let total_balance = account_balances
                .iter()
                .flat_map(|account| account.values())
                .sum::<u128>();
            if total_balance == 0 {
                let first_account = accounts
                    .get(&0)
                    .ok_or_else(|| anyhow::anyhow!("No account derived for this wallet"))?;
                let target = Target::from_str(&chain_info.name).unwrap_or_default();
                let message = if let Some(faucet_url) = target.faucet_url() {
                    format!(
                        "Your wallet does not have any funds to pay for the transaction.\
                        \n\nIf you are interacting with a testnet, consider using the faucet.\
                        \n-> {target} network faucet: {faucet_url}/?address={first_account}\
                        \nIf you are interacting with a local node, consider providing a chainConfig which funds your account."
                    )
                } else {
                    "Your wallet does not have any funds to pay for the transaction.".to_string()
                };
                anyhow::bail!(message)
            }

            // TODO: Do this via forc-wallet once the functionality is exposed.
            // TODO: calculate the number of transactions to sign and ask the user to confirm.
            let question = format!(
                "Do you agree to sign {tx_count} transaction{}?",
                if tx_count > 1 { "s" } else { "" }
            );
            let accepted = ask_user_yes_no_question(&question)?;
            if !accepted {
                anyhow::bail!("User refused to sign");
            }

            let wallet = select_local_wallet_account(password, provider).await?;
            Ok(ForcClientAccount::Wallet(wallet))
        }
        SignerSelectionMode::Manual => {
            let secret_key = select_manual_secret_key(default_sign, signing_key)
                .ok_or_else(|| anyhow::anyhow!("missing manual secret key"))?;
            let signer = PrivateKeySigner::new(secret_key);
            let wallet = Wallet::new(signer, provider.clone());
            Ok(ForcClientAccount::Wallet(wallet))
        }
        SignerSelectionMode::AwsSigner(arn) => {
            let aws_config = AwsConfig::from_env().await;
            let aws_client = AwsClient::new(aws_config);
            let aws_signer = AwsSigner::new(aws_client, arn.clone(), provider.clone()).await?;

            let account = ForcClientAccount::KmsSigner(aws_signer);
            Ok(account)
        }
    }
}

pub(crate) async fn select_local_wallet_account(
    password: &str,
    provider: &Provider,
) -> Result<Wallet<Unlocked<PrivateKeySigner>>> {
    let wallet_path = default_wallet_path();
    let accounts = collect_user_accounts(&wallet_path, password, provider.url()).await?;
    let account_balances = collect_account_balances(&accounts, provider).await?;
    let consensus_parameters = provider.consensus_parameters().await?;
    let base_asset_id = consensus_parameters.base_asset_id();
    let selections =
        format_base_asset_account_balances(&accounts, &account_balances, base_asset_id)?;

    let mut account_index;
    loop {
        account_index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Wallet account")
            .max_length(5)
            .items(&selections[..])
            .default(0)
            .interact()?;

        if accounts.contains_key(&account_index) {
            break;
        }
        let options: Vec<String> = accounts
            .keys()
            .map(|key| {
                let raw_addr = format!("0x{key}");
                let checksum_addr = checksum_encode(&raw_addr)?;
                Ok(checksum_addr)
            })
            .collect::<Result<Vec<_>>>()?;
        println_warning(&format!(
            "\"{}\" is not a valid account.\nPlease choose a valid option from {}",
            account_index,
            options.join(","),
        ));
    }

    let secret_key = secret_key_from_forc_wallet(&wallet_path, account_index, password)?;
    let signer = PrivateKeySigner::new(secret_key);
    let wallet = Wallet::new(signer, provider.clone());
    Ok(wallet)
}

pub async fn update_proxy_contract_target(
    account: &ForcClientAccount,
    proxy_contract_id: ContractId,
    new_target: ContractId,
) -> Result<CallResponse<()>> {
    abigen!(Contract(name = "ProxyContract", abi = "{\"programType\":\"contract\",\"specVersion\":\"1.1\",\"encodingVersion\":\"1\",\"concreteTypes\":[{\"type\":\"()\",\"concreteTypeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"type\":\"enum standards::src5::AccessError\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\",\"metadataTypeId\":1},{\"type\":\"enum standards::src5::State\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"metadataTypeId\":2},{\"type\":\"enum std::option::Option<struct std::contract_id::ContractId>\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"metadataTypeId\":4,\"typeArguments\":[\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\",\"metadataTypeId\":5},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\",\"metadataTypeId\":6},{\"type\":\"str\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"},{\"type\":\"struct std::contract_id::ContractId\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\",\"metadataTypeId\":9},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\",\"metadataTypeId\":10},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\",\"metadataTypeId\":11}],\"metadataTypes\":[{\"type\":\"b256\",\"metadataTypeId\":0},{\"type\":\"enum standards::src5::AccessError\",\"metadataTypeId\":1,\"components\":[{\"name\":\"NotOwner\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum standards::src5::State\",\"metadataTypeId\":2,\"components\":[{\"name\":\"Uninitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Initialized\",\"typeId\":3},{\"name\":\"Revoked\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum std::identity::Identity\",\"metadataTypeId\":3,\"components\":[{\"name\":\"Address\",\"typeId\":8},{\"name\":\"ContractId\",\"typeId\":9}]},{\"type\":\"enum std::option::Option\",\"metadataTypeId\":4,\"components\":[{\"name\":\"None\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Some\",\"typeId\":7}],\"typeParameters\":[7]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"metadataTypeId\":5,\"components\":[{\"name\":\"CannotReinitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"metadataTypeId\":6,\"components\":[{\"name\":\"CannotUninitialize\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"generic T\",\"metadataTypeId\":7},{\"type\":\"struct std::address::Address\",\"metadataTypeId\":8,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct std::contract_id::ContractId\",\"metadataTypeId\":9,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"metadataTypeId\":10,\"components\":[{\"name\":\"new_proxy_owner\",\"typeId\":2}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"metadataTypeId\":11,\"components\":[{\"name\":\"new_target\",\"typeId\":9}]}],\"functions\":[{\"inputs\":[],\"name\":\"proxy_target\",\"output\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [Option<ContractId>] - The new proxy contract to which all fallback calls will be passed or `None`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[{\"name\":\"new_target\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"}],\"name\":\"set_proxy_target\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Change the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called by the `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_target`: [ContractId] - The new proxy contract to which all fallback calls will be passed.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When not called by `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Write: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\",\"write\"]}]},{\"inputs\":[],\"name\":\"proxy_owner\",\"output\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the owner of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [State] - Represents the state of ownership for this contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[],\"name\":\"initialize_proxy\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Initializes the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method sets the storage values using the values of the configurable constants `INITIAL_TARGET` and `INITIAL_OWNER`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This then allows methods that write to storage to be called.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called once.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When `storage::SRC14.proxy_owner` is not [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `2`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]},{\"inputs\":[{\"name\":\"new_proxy_owner\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\"}],\"name\":\"set_proxy_owner\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Changes proxy ownership to the passed State.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can be used to transfer ownership between Identities or to revoke ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_proxy_owner`: [State] - The new state of the proxy ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the sender is not the current proxy owner.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the new state of the proxy ownership is [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]}],\"loggedTypes\":[{\"logId\":\"4571204900286667806\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\"},{\"logId\":\"2151606668983994881\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\"},{\"logId\":\"2161305517876418151\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\"},{\"logId\":\"4354576968059844266\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\"},{\"logId\":\"10870989709723147660\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\"},{\"logId\":\"10098701174489624218\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"}],\"messagesTypes\":[],\"configurables\":[{\"name\":\"INITIAL_TARGET\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"offset\":13368},{\"name\":\"INITIAL_OWNER\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"offset\":13320}]}",));

    let proxy_contract = ProxyContract::new(proxy_contract_id, account.clone());

    let result = proxy_contract
        .methods()
        .set_proxy_target(new_target)
        .call()
        .await?;
    println_action_green(
        "Updated",
        &format!("proxy contract target to 0x{new_target}"),
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_format_base_asset_account_balances() {
        let mut accounts_map: AccountsMap = BTreeMap::new();

        let address1 = fuel_tx::Address::from_str(
            "7bbd8a4ea06e94461b959ab18d35802bbac3cf47e2bf29195f7db2ce41630cd7",
        )
        .expect("address1");

        let address2 = fuel_tx::Address::from_str(
            "99bd8a4ea06e94461b959ab18d35802bbac3cf47e2bf29195f7db2ce41630cd7",
        )
        .expect("address2");

        let base_asset_id = AssetId::zeroed();

        accounts_map.insert(0, address1);
        accounts_map.insert(1, address2);

        let mut account_balances: AccountBalances = Vec::new();
        let mut balance1 = HashMap::new();
        balance1.insert(base_asset_id.to_string(), 1_500_000_000);
        balance1.insert("other_asset".to_string(), 2_000_000_000);
        account_balances.push(balance1);

        let mut balance2 = HashMap::new();
        balance2.insert("other_asset".to_string(), 3_000_000_000);
        account_balances.push(balance2);

        let address1_expected =
            "0x7bBD8a4ea06E94461b959aB18d35802BbAC3cf47e2bF29195F7db2CE41630CD7";
        let address2_expected =
            "0x99Bd8a4eA06E94461b959AB18d35802bBaC3Cf47E2Bf29195f7DB2cE41630cD7";
        let expected = vec![
            format!("[0] {address1_expected} - 1.5 ETH"),
            format!("[1] {address2_expected} - 0 ETH"),
        ];

        let result =
            format_base_asset_account_balances(&accounts_map, &account_balances, &base_asset_id)
                .unwrap();
        assert_eq!(result, expected);
    }
}
