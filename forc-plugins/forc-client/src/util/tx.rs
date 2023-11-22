use std::{io::Write, str::FromStr};

use anyhow::{Error, Result};
use async_trait::async_trait;
use forc_tracing::println_warning;
use fuel_crypto::{Message, PublicKey, SecretKey, Signature};
use fuel_tx::{
    field, Address, AssetId, Buildable, ContractId, Input, Output, TransactionBuilder, Witness,
};
use fuels_accounts::{provider::Provider, wallet::Wallet, ViewOnlyAccount};
use fuels_core::types::{
    bech32::{Bech32Address, FUEL_BECH32_HRP},
    coin_type::CoinType,
    transaction_builders::{create_coin_input, create_coin_message_input},
};

use forc_wallet::{
    account::{derive_secret_key, new_at_index_cli},
    balance::{
        collect_accounts_with_verification, print_account_balances, AccountBalances,
        AccountVerification, AccountsMap,
    },
    new::new_wallet_cli,
    utils::default_wallet_path,
};

use crate::constants::BETA_4_FAUCET_URL;

/// The maximum time to wait for a transaction to be included in a block by the node
pub const TX_SUBMIT_TIMEOUT_MS: u64 = 30_000u64;

/// Default PrivateKey to sign transactions submitted to local node.
pub const DEFAULT_PRIVATE_KEY: &str =
    "0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";

#[derive(PartialEq, Eq)]
pub enum WalletSelectionMode {
    ForcWallet,
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
    print!("{question}");
    std::io::stdout().flush()?;
    let mut ans = String::new();
    std::io::stdin().read_line(&mut ans)?;
    // Pop trailing \n as users press enter to submit their answers.
    ans.pop();
    // Trim the user input as it might have an additional space.
    let ans = ans.trim();
    Ok(ans == "y" || ans == "Y")
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

#[async_trait]
pub trait TransactionBuilderExt<Tx> {
    fn add_contract(&mut self, contract_id: ContractId) -> &mut Self;
    fn add_contracts(&mut self, contract_ids: Vec<ContractId>) -> &mut Self;
    fn add_inputs(&mut self, inputs: Vec<Input>) -> &mut Self;
    async fn fund(
        &mut self,
        address: Address,
        provider: Provider,
        signature_witness_index: u8,
    ) -> Result<&mut Self>;
    async fn finalize_signed(
        &mut self,
        client: Provider,
        unsigned: bool,
        signing_key: Option<SecretKey>,
        wallet_mode: WalletSelectionMode,
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
        signature_witness_index: u8,
    ) -> Result<&mut Self> {
        let wallet = Wallet::from_address(Bech32Address::from(address), Some(provider));

        let amount = 1_000_000;
        let asset_id = AssetId::BASE;
        let inputs: Vec<_> = wallet
            .get_spendable_resources(asset_id, amount)
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
        wallet_mode: WalletSelectionMode,
    ) -> Result<Tx> {
        let params = provider.chain_info().await?.consensus_parameters;
        let signing_key = match (wallet_mode, signing_key, default_sign) {
            (WalletSelectionMode::ForcWallet, None, false) => {
                // TODO: This is a very simple TUI, we should consider adding a nice TUI
                // capabilities for selections and answer collection.
                let wallet_path = default_wallet_path();
                if !wallet_path.exists() {
                    let question = format!("Could not find a wallet at {wallet_path:?}, would you like to create a new one? [y/N]: ");
                    let accepted = ask_user_yes_no_question(&question)?;
                    if accepted {
                        new_wallet_cli(&wallet_path)?;
                        println!("Wallet created successfully.");
                        // Derive first account for the fresh wallet we created.
                        new_at_index_cli(&wallet_path, 0)?;
                        println!("Account derived successfully.");
                    } else {
                        anyhow::bail!("Refused to create a new wallet. If you don't want to use forc-wallet, you can sign this transaction manually with --manual-signing flag.")
                    }
                }
                let prompt = format!(
                        "\nPlease provide the password of your encrypted wallet vault at {wallet_path:?}: "
                    );
                let password = rpassword::prompt_password(prompt)?;
                let verification = AccountVerification::Yes(password.clone());
                let accounts = collect_accounts_with_verification(&wallet_path, verification)?;
                let account_balances = collect_account_balances(&accounts, &provider).await?;

                let total_balance = account_balances
                    .iter()
                    .flat_map(|account| account.values())
                    .sum::<u64>();
                if total_balance == 0 {
                    let first_account = accounts
                        .get(&0)
                        .ok_or_else(|| anyhow::anyhow!("No account derived for this wallet"))?;
                    let faucet_link = format!("{}/?address={first_account}", BETA_4_FAUCET_URL);
                    anyhow::bail!("Your wallet does not have any funds to pay for the transaction.\
                                      \n\nIf you are interacting with a testnet consider using the faucet.\
                                      \n-> beta-4 network faucet: {faucet_link}\
                                      \nIf you are interacting with a local node, consider providing a chainConfig which funds your account.")
                }
                print_account_balances(&accounts, &account_balances);

                print!("\nPlease provide the index of account to use for signing: ");
                std::io::stdout().flush()?;
                let mut account_index = String::new();
                std::io::stdin().read_line(&mut account_index)?;
                let account_index = account_index.trim().parse::<usize>()?;

                let secret_key = derive_secret_key(&wallet_path, account_index, &password)
                    .map_err(|e| {
                        if e.to_string().contains("Mac Mismatch") {
                            anyhow::anyhow!(
                                "Failed to access forc-wallet vault. Please check your password"
                            )
                        } else {
                            e
                        }
                    })?;

                // TODO: Do this via forc-wallet once the functinoality is exposed.
                let public_key = PublicKey::from(&secret_key);
                let hashed = public_key.hash();
                let bech32 = Bech32Address::new(FUEL_BECH32_HRP, hashed);
                let question = format!(
                    "Do you agree to sign this transaction with {}? [y/N]: ",
                    bech32
                );
                let accepted = ask_user_yes_no_question(&question)?;
                if !accepted {
                    anyhow::bail!("User refused to sign");
                }

                Some(secret_key)
            }
            (WalletSelectionMode::ForcWallet, Some(key), _) => {
                println_warning("Signing key is provided while requesting to sign with forc-wallet or with default signer. Using signing key");
                Some(key)
            }
            (WalletSelectionMode::Manual, None, false) => None,
            (WalletSelectionMode::Manual, Some(key), false) => Some(key),
            (_, None, true) => {
                // Generate a `SecretKey` to sign this transaction from a default private key used
                // by fuel-core.
                let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY)?;
                Some(secret_key)
            }
            (WalletSelectionMode::Manual, Some(key), true) => {
                println_warning("Signing key is provided while requesting to sign with a default signer. Using signing key");
                Some(key)
            }
        };
        // Get the address
        let address = if let Some(key) = signing_key {
            Address::from(*key.public_key().hash())
        } else {
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
            let message = Message::from_bytes(*tx.id(&params.chain_id));
            Signature::sign(&signing_key, &message)
        } else {
            prompt_signature(tx.id(&params.chain_id))?
        };

        let witness = Witness::from(signature.as_ref());
        tx.replace_witness(signature_witness_index, witness);
        tx.precompute(&params.chain_id).unwrap(); // TODO: make error conversion

        Ok(tx)
    }
}

pub trait TransactionExt {
    fn replace_witness(&mut self, witness_index: u8, witness: Witness) -> &mut Self;
}

impl<T: field::Witnesses> TransactionExt for T {
    fn replace_witness(&mut self, index: u8, witness: Witness) -> &mut Self {
        self.witnesses_mut()[index as usize] = witness;
        self
    }
}
