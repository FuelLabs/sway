use fuel_crypto::{Message, Signature};
use fuels::{
    prelude::*,
    types::{coin_type_id::CoinTypeId, input::Input},
};
use fuels_accounts::{wallet::WalletUnlocked, Account};

/// Accounts that can be used with forc-client.
#[derive(Clone, Debug)]
pub enum ForcClientAccount {
    Wallet(WalletUnlocked),
    KmsSigner,
}

impl Account for ForcClientAccount {
    /// Returns a vector consisting of `Input::Coin`s and `Input::Message`s for the given
    /// asset ID and amount. The `witness_index` is the position of the witness (signature)
    /// in the transaction\'s list of witnesses. In the validation process, the node will
    /// use the witness at this index to validate the coins returned by this method.
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn get_asset_inputs_for_amount<'life0, 'async_trait>(
        &'life0 self,
        asset_id: AssetId,
        amount: u64,
        excluded_coins: Option<Vec<CoinTypeId>>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Vec<Input>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        match self {
            ForcClientAccount::Wallet(wallet) => {
                wallet.get_asset_inputs_for_amount(asset_id, amount, excluded_coins)
            }
            ForcClientAccount::KmsSigner => todo!(),
        }
    }
}

impl ViewOnlyAccount for ForcClientAccount {
    fn address(&self) -> &Bech32Address {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.address(),
            ForcClientAccount::KmsSigner => todo!(),
        }
    }

    fn try_provider(&self) -> Result<&Provider> {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.try_provider(),
            ForcClientAccount::KmsSigner => todo!(),
        }
    }
}

impl Signer for ForcClientAccount {
    fn sign<'life0, 'async_trait>(
        &'life0 self,
        message: Message,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Signature>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.sign(message),
            ForcClientAccount::KmsSigner => todo!(),
        }
    }

    fn address(&self) -> &Bech32Address {
        match self {
            ForcClientAccount::Wallet(wallet) => wallet.address(),
            ForcClientAccount::KmsSigner => todo!(),
        }
    }
}
