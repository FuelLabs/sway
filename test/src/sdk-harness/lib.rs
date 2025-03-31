use fuels::{
    accounts::{
        keystore::Keystore,
        signers::{ private_key::PrivateKeySigner},
    },
    crypto::SecretKey,
    prelude::*,
};
use rand::thread_rng;

pub(crate) async fn new_random_wallet(provider: Option<Provider>) -> Wallet {
    let mut rng = thread_rng();
    let signer = PrivateKeySigner::random(&mut rng);
    let provider = match provider {
        Some(provider) => provider,
        None => setup_test_provider(vec![], vec![], None, None).await?,
    };
    Wallet::new(signer, provider.clone());
}