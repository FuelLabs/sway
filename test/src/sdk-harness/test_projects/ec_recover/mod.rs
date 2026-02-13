use fuel_vm::{
    fuel_crypto::{Message, PublicKey, SecretKey, Signature},
    fuel_tx::Bytes64,
    fuel_types::Bytes32,
};
use fuels::{accounts::signers::private_key::PrivateKeySigner, prelude::*, types::Bits256};
use rand::{rngs::StdRng, Rng, SeedableRng};

abigen!(Contract(
    name = "EcRecoverContract",
    abi = "out_for_sdk_harness_tests/ec_recover-abi.json"
));

async fn setup_env() -> Result<(
    EcRecoverContract<Wallet>,
    SecretKey,
    PublicKey,
    Wallet,
    Message,
    Bytes64,
)> {
    let mut rng = StdRng::seed_from_u64(1000);
    let msg_bytes: Bytes32 = rng.r#gen();
    let private_key = SecretKey::random(&mut rng);
    let public_key = PublicKey::from(&private_key);
    let msg = Message::from_bytes(*msg_bytes);
    let sig = Signature::sign(&private_key, &msg);
    let sig_bytes: Bytes64 = Bytes64::from(sig);
    let signer = PrivateKeySigner::new(private_key);

    let num_assets = 1;
    let coins_per_asset = 10;
    let amount_per_coin = 15;
    let (coins, _asset_ids) = setup_multiple_assets_coins(
        signer.address(),
        num_assets,
        coins_per_asset,
        amount_per_coin,
    );
    let provider = setup_test_provider(coins.clone(), vec![], None, None)
        .await
        .unwrap();
    let wallet = Wallet::new(signer, provider);

    let contract_id = Contract::load_from(
        "out_for_sdk_harness_tests/ec_recover.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let contract_instance = EcRecoverContract::new(contract_id, wallet.clone());

    Ok((
        contract_instance,
        private_key,
        public_key,
        wallet,
        msg,
        sig_bytes,
    ))
}

#[tokio::test]
async fn can_recover_public_key() {
    let (contract, _secret, public_key, _wallet, msg, sig_bytes) = setup_env().await.unwrap();
    let sig_r = &sig_bytes[..32];
    let sig_v_s = &sig_bytes[32..];
    let response = contract
        .methods()
        .recover_pub_key(
            Bits256(sig_r.try_into().unwrap()),
            Bits256(sig_v_s.try_into().unwrap()),
            Bits256(msg.into()),
        )
        .call()
        .await
        .unwrap();

    let first = response.value.0;
    let second = response.value.1;
    let arrays: [[u8; 32]; 2] = [first.0, second.0];
    let joined: Vec<u8> = arrays.into_iter().flat_map(|s| s.into_iter()).collect();
    let joined_array: [u8; 64] = joined.try_into().unwrap();
    let pubkey = Bytes64::new(joined_array);

    assert_eq!(pubkey, Bytes64::new(*public_key));
}

#[tokio::test]
async fn can_recover_address() {
    let (contract, _secret, _public_key, wallet, msg, sig_bytes) = setup_env().await.unwrap();
    let sig_r = &sig_bytes[..32];
    let sig_v_s = &sig_bytes[32..];
    let response = contract
        .methods()
        .recover_address(
            Bits256(sig_r.try_into().unwrap()),
            Bits256(sig_v_s.try_into().unwrap()),
            Bits256(*msg),
        )
        .call()
        .await
        .unwrap();

    assert_eq!(Address::from(response.value), wallet.address());
}
