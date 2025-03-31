use fuels::{
    accounts::{
        signers::private_key::PrivateKeySigner,
        wallet::Wallet,
    },
    prelude::*,
};
use rand::thread_rng;

pub async fn new_random_wallet(provider: Option<Provider>) -> Wallet {
    let signer = new_random_signer();
    let provider = match provider {
        Some(provider) => provider,
        None => setup_test_provider(vec![], vec![], None, None).await.expect("Failed to setup test provider"),
    };
    Wallet::new(signer, provider.clone())
}

pub fn new_random_signer() -> PrivateKeySigner {
    let mut rng = thread_rng();
    PrivateKeySigner::random(&mut rng)
}