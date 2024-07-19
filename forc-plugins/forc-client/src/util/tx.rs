use std::{collections::BTreeMap, io::Write, path::Path, str::FromStr};

use anyhow::{Error, Result};
use async_trait::async_trait;
use forc_tracing::println_warning;

use dialoguer::{theme::ColorfulTheme, Confirm, Password, Select};
use forc_wallet::{
    account::{derive_secret_key, new_at_index_cli},
    balance::{
        collect_accounts_with_verification, AccountBalances, AccountVerification, AccountsMap,
    },
    new::{new_wallet_cli, New},
    utils::default_wallet_path,
};
use fuel_crypto::{Message, SecretKey, Signature};
use fuel_tx::{
    field, Address, AssetId, Buildable, ContractId, Input, Output, TransactionBuilder, Witness,
};
use fuels_accounts::{provider::Provider, wallet::Wallet, ViewOnlyAccount};
use fuels_core::types::{
    bech32::Bech32Address,
    coin_type::CoinType,
    transaction_builders::{create_coin_input, create_coin_message_input},
};

use crate::{constants::DEFAULT_PRIVATE_KEY, util::target::Target};

#[derive(PartialEq, Eq)]
pub enum WalletSelectionMode {
    /// Holds the password of forc-wallet instance.
    ForcWallet(String),
    Manual,
}

fn prompt_address() -> Result<Bech32Address> {
    print!("Please provide the address of the wallet you are going to sign this transaction with:");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Bech32Address::from_str(buf.trim()).map_err(Error::msg)
}

fn prompt_signature(tx_id: fuel_tx::Bytes32) -> Result<Signature> {
    println!("Transaction id to sign: {tx_id}");
    print!("Please provide the signature:");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Signature::from_str(buf.trim()).map_err(Error::msg)
}

fn ask_user_yes_no_question(question: &str) -> Result<bool> {
    let answer = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(false)
        .show_default(false)
        .interact()?;
    Ok(answer)
}

fn collect_user_accounts(
    wallet_path: &Path,
    password: &str,
) -> Result<BTreeMap<usize, Bech32Address>> {
    let verification = AccountVerification::Yes(password.to_string());
    let accounts = collect_accounts_with_verification(wallet_path, verification).map_err(|e| {
        if e.to_string().contains("Mac Mismatch") {
            anyhow::anyhow!("Failed to access forc-wallet vault. Please check your password")
        } else {
            e
        }
    })?;
    Ok(accounts)
}

pub(crate) fn prompt_forc_wallet_password() -> Result<String> {
    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Wallet password")
        .allow_empty_password(true)
        .interact()?;

    Ok(password)
}

pub(crate) fn check_and_create_wallet_at_default_path(wallet_path: &Path) -> Result<()> {
    if !wallet_path.exists() {
        let question = format!("Could not find a wallet at {wallet_path:?}, would you like to create a new one? [y/N]: ");
        let accepted = ask_user_yes_no_question(&question)?;
        let new_options = New {
            force: false,
            cache_accounts: None,
        };
        if accepted {
            new_wallet_cli(wallet_path, new_options)?;
            println!("Wallet created successfully.");
            // Derive first account for the fresh wallet we created.
            new_at_index_cli(wallet_path, 0)?;
            println!("Account derived successfully.");
        } else {
            anyhow::bail!("Refused to create a new wallet. If you don't want to use forc-wallet, you can sign this transaction manually with --manual-signing flag.")
        }
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
    Ok(secret_key)
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
        .map(|addr| Wallet::from_address(addr.clone(), Some(provider.clone())))
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
) -> Vec<String> {
    accounts_map
        .iter()
        .zip(account_balances)
        .map(|((ix, address), balance)| {
            let base_asset_amount = balance
                .get(&base_asset_id.to_string())
                .copied()
                .unwrap_or(0);
            let eth_amount = base_asset_amount as f64 / 1_000_000_000.0;
            format!("[{ix}] {address} - {eth_amount} ETH")
        })
        .collect()
}

// TODO: Simplify the function signature once https://github.com/FuelLabs/sway/issues/6071 is closed.
pub(crate) async fn select_secret_key(
    wallet_mode: &WalletSelectionMode,
    default_sign: bool,
    signing_key: Option<SecretKey>,
    provider: &Provider,
    tx_count: usize,
) -> Result<Option<SecretKey>> {
    let chain_info = provider.chain_info().await?;
    let signing_key = match wallet_mode {
        WalletSelectionMode::ForcWallet(password) => {
            let wallet_path = default_wallet_path();
            check_and_create_wallet_at_default_path(&wallet_path)?;
            // TODO: This is a very simple TUI, we should consider adding a nice TUI
            // capabilities for selections and answer collection.
            let accounts = collect_user_accounts(&wallet_path, password)?;
            let account_balances = collect_account_balances(&accounts, provider).await?;
            let base_asset_id = provider.base_asset_id();

            let total_balance = account_balances
                .iter()
                .flat_map(|account| account.values())
                .sum::<u64>();
            if total_balance == 0 {
                let first_account = accounts
                    .get(&0)
                    .ok_or_else(|| anyhow::anyhow!("No account derived for this wallet"))?;
                let target = Target::from_str(&chain_info.name).unwrap_or(Target::testnet());
                let faucet_link = format!("{}/?address={first_account}", target.faucet_url());
                anyhow::bail!("Your wallet does not have any funds to pay for the transaction.\
                                      \n\nIf you are interacting with a testnet consider using the faucet.\
                                      \n-> {target} network faucet: {faucet_link}\
                                      \nIf you are interacting with a local node, consider providing a chainConfig which funds your account.")
            }
            let selections =
                format_base_asset_account_balances(&accounts, &account_balances, base_asset_id);

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
                let options: Vec<String> = accounts.keys().map(|key| key.to_string()).collect();
                println_warning(&format!(
                    "\"{}\" is not a valid account.\nPlease choose a valid option from {}",
                    account_index,
                    options.join(","),
                ));
            }

            let secret_key = secret_key_from_forc_wallet(&wallet_path, account_index, password)?;

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

            Some(secret_key)
        }
        WalletSelectionMode::Manual => select_manual_secret_key(default_sign, signing_key),
    };
    Ok(signing_key)
}

#[async_trait]
pub trait TransactionBuilderExt<Tx> {
    fn add_contract(&mut self, contract_id: ContractId) -> &mut Self;
    fn add_contracts(&mut self, contract_ids: Vec<ContractId>) -> &mut Self;
    fn add_inputs(&mut self, inputs: Vec<Input>) -> &mut Self;
    async fn fund(
        &mut self,
        address: Address,
        provider: Provider,
        signature_witness_index: u16,
    ) -> Result<&mut Self>;
    async fn finalize_signed(
        &mut self,
        client: Provider,
        default_signature: bool,
        signing_key: Option<SecretKey>,
        wallet_mode: &WalletSelectionMode,
    ) -> Result<Tx>;
}

#[async_trait]
impl<Tx: Buildable + field::Witnesses + Send> TransactionBuilderExt<Tx> for TransactionBuilder<Tx> {
    fn add_contract(&mut self, contract_id: ContractId) -> &mut Self {
        let input_index = self
            .inputs()
            .len()
            .try_into()
            .expect("limit of 256 inputs exceeded");
        self.add_input(fuel_tx::Input::contract(
            fuel_tx::UtxoId::new(fuel_tx::Bytes32::zeroed(), 0),
            fuel_tx::Bytes32::zeroed(),
            fuel_tx::Bytes32::zeroed(),
            fuel_tx::TxPointer::new(0u32.into(), 0),
            contract_id,
        ))
        .add_output(fuel_tx::Output::Contract(
            fuel_tx::output::contract::Contract {
                input_index,
                balance_root: fuel_tx::Bytes32::zeroed(),
                state_root: fuel_tx::Bytes32::zeroed(),
            },
        ))
    }
    fn add_contracts(&mut self, contract_ids: Vec<ContractId>) -> &mut Self {
        for contract_id in contract_ids {
            self.add_contract(contract_id);
        }
        self
    }
    fn add_inputs(&mut self, inputs: Vec<Input>) -> &mut Self {
        for input in inputs {
            self.add_input(input);
        }
        self
    }
    async fn fund(
        &mut self,
        address: Address,
        provider: Provider,
        signature_witness_index: u16,
    ) -> Result<&mut Self> {
        let asset_id = *provider.base_asset_id();
        let wallet = Wallet::from_address(Bech32Address::from(address), Some(provider));

        let amount = 1_000_000;
        let filter = None;
        let inputs: Vec<_> = wallet
            .get_spendable_resources(asset_id, amount, filter)
            .await?
            .into_iter()
            .map(|coin_type| match coin_type {
                CoinType::Coin(coin) => create_coin_input(coin, signature_witness_index),
                CoinType::Message(message) => {
                    create_coin_message_input(message, signature_witness_index)
                }
            })
            .collect();
        let output = Output::change(wallet.address().into(), 0, asset_id);

        self.add_inputs(inputs).add_output(output);

        Ok(self)
    }
    async fn finalize_signed(
        &mut self,
        provider: Provider,
        default_sign: bool,
        signing_key: Option<SecretKey>,
        wallet_mode: &WalletSelectionMode,
    ) -> Result<Tx> {
        let chain_info = provider.chain_info().await?;
        let params = chain_info.consensus_parameters;
        let signing_key =
            select_secret_key(wallet_mode, default_sign, signing_key, &provider, 1).await?;
        // Get the address
        let address = if let Some(key) = signing_key {
            Address::from(*key.public_key().hash())
        } else {
            // TODO: Remove this path https://github.com/FuelLabs/sway/issues/6071
            Address::from(prompt_address()?)
        };

        // Insert dummy witness for signature
        let signature_witness_index = self.witnesses().len().try_into()?;
        self.add_witness(Witness::default());

        // Add input coin and output change
        self.fund(
                address,
                provider,
                signature_witness_index,
            )
            .await.map_err(|e| if e.to_string().contains("not enough coins to fit the target") {
                anyhow::anyhow!("Deployment failed due to insufficient funds. Please be sure to have enough coins to pay for deployment transaction.")
            } else {
                e
            })?;

        let mut tx = self.finalize_without_signature_inner();

        let signature = if let Some(signing_key) = signing_key {
            let message = Message::from_bytes(*tx.id(&params.chain_id()));
            Signature::sign(&signing_key, &message)
        } else {
            prompt_signature(tx.id(&params.chain_id()))?
        };

        let witness = Witness::from(signature.as_ref());
        tx.replace_witness(signature_witness_index, witness);
        tx.precompute(&params.chain_id())
            .map_err(anyhow::Error::msg)?;

        Ok(tx)
    }
}

pub trait TransactionExt {
    fn replace_witness(&mut self, witness_index: u16, witness: Witness) -> &mut Self;
}

impl<T: field::Witnesses> TransactionExt for T {
    fn replace_witness(&mut self, index: u16, witness: Witness) -> &mut Self {
        self.witnesses_mut()[index as usize] = witness;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::collections::HashMap;

    #[test]
    fn test_format_base_asset_account_balances() {
        let mut accounts_map: AccountsMap = BTreeMap::new();

        let address1 = Bech32Address::from_str(
            "fuel1dved7k25uxadatl7l5kql309jnw07dcn4t3a6x9hm9nxyjcpqqns50p7n2",
        )
        .expect("address1");
        let address2 = Bech32Address::from_str(
            "fuel1x9f3ysyk7fmey5ac23s2p4rwg4gjye2kke3nu3pvrs5p4qc4m4qqwx56k3",
        )
        .expect("address2");

        let base_asset_id = AssetId::zeroed();

        accounts_map.insert(0, address1.clone());
        accounts_map.insert(1, address2.clone());

        let mut account_balances: AccountBalances = Vec::new();
        let mut balance1 = HashMap::new();
        balance1.insert(base_asset_id.to_string(), 1_500_000_000);
        balance1.insert("other_asset".to_string(), 2_000_000_000);
        account_balances.push(balance1);

        let mut balance2 = HashMap::new();
        balance2.insert("other_asset".to_string(), 3_000_000_000);
        account_balances.push(balance2);

        let expected = vec![
            format!("[0] {address1} - 1.5 ETH"),
            format!("[1] {address2} - 0 ETH"),
        ];

        let result =
            format_base_asset_account_balances(&accounts_map, &account_balances, &base_asset_id);
        assert_eq!(result, expected);
    }
}
