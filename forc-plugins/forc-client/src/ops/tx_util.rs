use std::{io::Write, str::FromStr};

use anyhow::{Error, Result};
use async_trait::async_trait;
use fuel_gql_client::{
    client::FuelClient,
    fuel_crypto::{Message, SecretKey, Signature},
    fuel_tx::{Address, ContractId, Input, Output, TransactionBuilder, Witness},
    prelude::SerializableVec,
};
use fuel_tx::{field, Buildable};
use fuels_core::constants::BASE_ASSET_ID;
use fuels_signers::{provider::Provider, Wallet};
use fuels_types::bech32::Bech32Address;

/// The maximum time to wait for a transaction to be included in a block by the node
pub const TX_SUBMIT_TIMEOUT_MS: u64 = 30_000u64;

fn prompt_address() -> Result<Bech32Address> {
    print!("Please provide the address of the wallet you are going to sign this transaction with:");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Bech32Address::from_str(buf.trim()).map_err(Error::msg)
}

fn prompt_signature(tx_id: fuel_tx::Bytes32) -> Result<Signature> {
    println!("Transaction id to sign: {}", tx_id);
    print!("Please provide the signature:");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Signature::from_str(buf.trim()).map_err(Error::msg)
}

#[derive(Debug)]
pub struct TxParameters {
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl TxParameters {
    pub const DEFAULT: Self = Self {
        gas_limit: fuel_tx::ConsensusParameters::DEFAULT.max_gas_per_tx,
        gas_price: 0,
    };

    pub fn new(gas_limit: Option<u64>, gas_price: Option<u64>) -> Self {
        Self {
            gas_limit: gas_limit.unwrap_or(TxParameters::DEFAULT.gas_limit),
            gas_price: gas_price.unwrap_or(TxParameters::DEFAULT.gas_price),
        }
    }
}

impl Default for TxParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[async_trait]
pub trait TransactionBuilderExt<Tx> {
    fn params(&mut self, params: TxParameters) -> &mut Self;
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
    ) -> Result<Tx>;
}

#[async_trait]
impl<Tx: Buildable + SerializableVec + field::Witnesses + Send> TransactionBuilderExt<Tx>
    for TransactionBuilder<Tx>
{
    fn params(&mut self, params: TxParameters) -> &mut Self {
        self.gas_limit(params.gas_limit).gas_price(params.gas_price)
    }
    fn add_contract(&mut self, contract_id: ContractId) -> &mut Self {
        let input_index = self
            .inputs()
            .len()
            .try_into()
            .expect("limit of 256 inputs exceeded");
        self.add_input(fuel_tx::Input::Contract {
            contract_id,
            utxo_id: fuel_tx::UtxoId::new(fuel_tx::Bytes32::zeroed(), 0),
            balance_root: fuel_tx::Bytes32::zeroed(),
            state_root: fuel_tx::Bytes32::zeroed(),
            tx_pointer: fuel_tx::TxPointer::new(0, 0),
        })
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
        let asset_id = BASE_ASSET_ID;
        let inputs = wallet
            .get_asset_inputs_for_amount(asset_id, amount, signature_witness_index)
            .await?;
        let output = Output::change(wallet.address().into(), 0, asset_id);

        self.add_inputs(inputs).add_output(output);

        Ok(self)
    }
    async fn finalize_signed(
        &mut self,
        client: FuelClient,
        unsigned: bool,
        signing_key: Option<SecretKey>,
    ) -> Result<Tx> {
        let mut signature_witness_index = 0u8;
        if !unsigned {
            // Get the address
            let address = if let Some(signing_key) = signing_key {
                Address::from(*signing_key.public_key().hash())
            } else {
                Address::from(prompt_address()?)
            };

            // Insert dummy witness for signature
            signature_witness_index = self.witnesses().len().try_into()?;
            self.add_witness(Witness::default());

            // Add input coin and output change
            self.fund(address, Provider::new(client), signature_witness_index)
                .await?;
        }

        let mut tx = self._finalize_without_signature();

        if !unsigned {
            let signature = if let Some(signing_key) = signing_key {
                // Safety: `Message::from_bytes_unchecked` is unsafe because
                // it can't guarantee that the provided bytes will be the product
                // of a cryptographically secure hash. However, the bytes are
                // coming from `tx.id()`, which already uses `Hasher::hash()`
                // to hash it using a secure hash mechanism.
                let message = unsafe { Message::from_bytes_unchecked(*tx.id()) };
                Signature::sign(&signing_key, &message)
            } else {
                prompt_signature(tx.id())?
            };

            let witness = Witness::from(signature.as_ref());
            tx.replace_witness(signature_witness_index, witness);
        }
        tx.precompute();

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
