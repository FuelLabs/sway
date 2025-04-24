use async_trait::async_trait;
use fuel_crypto::{Message, Signature};
use fuels::{
    prelude::*,
    types::{coin_type_id::CoinTypeId, input::Input},
};
use fuels_accounts::{
    signers::private_key::PrivateKeySigner,
    wallet::{Unlocked, Wallet},
    Account,
};

use super::aws::AwsSigner;

#[derive(Clone, Debug)]
/// Set of different signers available to be used with `forc-client` operations.
pub enum ForcClientAccount {
    /// Local signer where the private key owned locally. This can be
    /// generated through `forc-wallet` integration or manually by providing
    /// a private-key.
    Wallet(Wallet<Unlocked<PrivateKeySigner>>),
    /// A KMS Signer specifically using AWS KMS service. The signing key
    /// is managed by another entity for KMS signers. Messages are
    /// signed by the KMS entity. Signed transactions are retrieved
    /// and submitted to the node by `forc-client`.
    KmsSigner(AwsSigner),
}

impl Account for ForcClientAccount {
    fn add_witnesses<Tb: TransactionBuilder>(&self, tb: &mut Tb) -> Result<()> {
        tb.add_signer(self.clone())?;

        Ok(())
    }
}

#[async_trait]
impl ViewOnlyAccount for ForcClientAccount {
    fn address(&self) -> &Bech32Address {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.address(),
            ForcClientAccount::KmsSigner(account) => {
                fuels_accounts::ViewOnlyAccount::address(account)
            }
        }
    }

    fn try_provider(&self) -> Result<&Provider> {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.try_provider(),
            ForcClientAccount::KmsSigner(account) => Ok(account.provider()),
        }
    }

    async fn get_asset_inputs_for_amount(
        &self,
        asset_id: AssetId,
        amount: u128,
        excluded_coins: Option<Vec<CoinTypeId>>,
    ) -> Result<Vec<Input>> {
        match self {
            ForcClientAccount::Wallet(wallet) => {
                wallet
                    .get_asset_inputs_for_amount(asset_id, amount, excluded_coins)
                    .await
            }
            ForcClientAccount::KmsSigner(account) => {
                account
                    .get_asset_inputs_for_amount(asset_id, amount, excluded_coins)
                    .await
            }
        }
    }
}

#[async_trait]
impl Signer for ForcClientAccount {
    async fn sign(&self, message: Message) -> Result<Signature> {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.signer().sign(message).await,
            ForcClientAccount::KmsSigner(account) => account.sign(message).await,
        }
    }

    fn address(&self) -> &Bech32Address {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.address(),
            ForcClientAccount::KmsSigner(account) => fuels_core::traits::Signer::address(account),
        }
    }
}
