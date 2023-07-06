use std::{io::Write, str::FromStr};

use anyhow::{Error, Result};
use async_trait::async_trait;
use fuel_core_client::client::FuelClient;
use fuel_crypto::{Message, PublicKey, SecretKey, Signature};
use fuel_tx::{
    field, Address, AssetId, Buildable, ContractId, Input, Output, TransactionBuilder, Witness,
};
use fuel_vm::prelude::SerializableVec;
use fuels_accounts::{provider::Provider, wallet::Wallet, ViewOnlyAccount};
use fuels_core::types::{
    bech32::{Bech32Address, FUEL_BECH32_HRP},
    coin_type::CoinType,
    transaction_builders::{create_coin_input, create_coin_message_input},
};

use forc_wallet::{account::derive_secret_key, utils::default_wallet_path};

/// The maximum time to wait for a transaction to be included in a block by the node
pub const TX_SUBMIT_TIMEOUT_MS: u64 = 30_000u64;

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
        client: FuelClient,
        unsigned: bool,
        signing_key: Option<SecretKey>,
        wallet_mode: WalletSelectionMode,
    ) -> Result<Tx>;
}

#[async_trait]
impl<Tx: Buildable + SerializableVec + field::Witnesses + Send> TransactionBuilderExt<Tx>
    for TransactionBuilder<Tx>
{
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
        .add_output(fuel_tx::Output::Contract {
            input_index,
            balance_root: fuel_tx::Bytes32::zeroed(),
            state_root: fuel_tx::Bytes32::zeroed(),
        })
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
        client: FuelClient,
        unsigned: bool,
        signing_key: Option<SecretKey>,
        wallet_mode: WalletSelectionMode,
    ) -> Result<Tx> {
        let params = client.chain_info().await?.consensus_parameters.into();
        let mut signature_witness_index = 0u8;
        let signing_key = if !unsigned {
            let key = match (wallet_mode, signing_key) {
                (WalletSelectionMode::ForcWallet, None) => {
                    // TODO: This is a very simple TUI, we should consider adding a nice TUI
                    // capabilities for selections and answer collection.
                    let wallet_path = default_wallet_path();
                    if !wallet_path.exists() {
                        anyhow::bail!("Cannot find a wallet at {wallet_path:?}\nPlease a generate new wallet with `forc wallet new`");
                    }
                    let prompt = format!(
                        "\nPlease provide the password of your encrypted wallet vault at {wallet_path:?}:"
                    );
                    let password = rpassword::prompt_password(prompt)?;
                    // TODO: List all derived wallets via forc-wallet and let the users choose
                    // account.
                    let account_index = 0;
                    let secret_key = derive_secret_key(&wallet_path, account_index, &password).map_err(|e| {
                        if e.to_string().contains("Mac Mismatch") {
                            anyhow::anyhow!("Failed to access forc-wallet vault. Please check your password")
                        }else {
                            e
                        }
                    })?;

                    // TODO: Do this via forc-wallet once the functinoality is exposed.
                    let public_key = PublicKey::from(&secret_key);
                    let hashed = public_key.hash();
                    let bech32 = Bech32Address::new(FUEL_BECH32_HRP, hashed);
                    // TODO: Check for balance and suggest using the faucet.
                    print!("Do you accept to sign this transaction with {bech32} [y/N]:");
                    std::io::stdout().flush()?;
                    let mut ans = String::new();
                    std::io::stdin().read_line(&mut ans)?;
                    // Pop trailing \n as users press enter to submit their answers.
                    ans.pop();
                    // Trim the user input as it might have an additional space.
                    let ans = ans.trim();
                    if ans != "y" && ans != "Y" {
                        anyhow::bail!("User refused to sign");
                    }

                    Some(secret_key)
                }
                (WalletSelectionMode::ForcWallet, Some(key)) => {
                    tracing::warn!(
                        "Signing key is provided while requesting to sign with forc-wallet. Using signing key"
                    );
                    Some(key)
                }
                (WalletSelectionMode::Manual, None) => None,
                (WalletSelectionMode::Manual, Some(key)) => Some(key),
            };
            // Get the address
            let address = if let Some(key) = key {
                Address::from(*key.public_key().hash())
            } else {
                Address::from(prompt_address()?)
            };

            // Insert dummy witness for signature
            signature_witness_index = self.witnesses().len().try_into()?;
            self.add_witness(Witness::default());

            // Add input coin and output change
            self.fund(
                address,
                Provider::new(client, params),
                signature_witness_index,
            )
            .await?;
            key
        } else {
            None
        };

        let mut tx = self._finalize_without_signature();

        if !unsigned {
            let signature = if let Some(signing_key) = signing_key {
                // Safety: `Message::from_bytes_unchecked` is unsafe because
                // it can't guarantee that the provided bytes will be the product
                // of a cryptographically secure hash. However, the bytes are
                // coming from `tx.id()`, which already uses `Hasher::hash()`
                // to hash it using a secure hash mechanism.
                let message = Message::from_bytes(*tx.id(&params));
                Signature::sign(&signing_key, &message)
            } else {
                prompt_signature(tx.id(&params))?
            };

            let witness = Witness::from(signature.as_ref());
            tx.replace_witness(signature_witness_index, witness);
        }
        tx.precompute(&params);

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
