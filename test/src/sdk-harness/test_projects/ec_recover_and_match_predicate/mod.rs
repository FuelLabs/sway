use fuels::{
    accounts::{predicate::Predicate, signers::private_key::PrivateKeySigner},
    crypto::Message,
    prelude::*,
    types::B512,
};

abigen!(
    Predicate(
        name = "TestPredicate",
        abi = "out/ec_recover_and_match_predicate-abi.json"
    )
);

#[tokio::test]
async fn ec_recover_and_match_predicate_test() -> Result<()> {
    use fuel_vm::fuel_crypto::SecretKey;

    let secret_key1: SecretKey =
        "0x862512a2363db2b3a375c0d4bbbd27172180d89f23f2e259bac850ab02619301"
            .parse()
            .unwrap();

    let secret_key2: SecretKey =
        "0x37fa81c84ccd547c30c176b118d5cb892bdb113e8e80141f266519422ef9eefd"
            .parse()
            .unwrap();

    let secret_key3: SecretKey =
        "0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb"
            .parse()
            .unwrap();

    let signer_1 = PrivateKeySigner::new(secret_key1);
    let signer_2 = PrivateKeySigner::new(secret_key2);
    let signer_3 = PrivateKeySigner::new(secret_key3);

    let all_coins = [signer_1.address(), signer_2.address(), signer_3.address()]
        .iter()
        .flat_map(|wallet| setup_single_asset_coins(*wallet, AssetId::default(), 10, 1_000_000))
        .collect::<Vec<_>>();

    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let provider = setup_test_provider(all_coins, vec![], Some(node_config), None)
        .await
        .unwrap();
    let wallet_1 = Wallet::new(signer_1, provider.clone());
    let wallet_2 = Wallet::new(signer_2, provider.clone());
    let wallet_3 = Wallet::new(signer_3, provider.clone());

    let random_secret_key = SecretKey::random(&mut rand::thread_rng());
    let receiver = Wallet::new(PrivateKeySigner::new(random_secret_key), provider.clone());

    let data_to_sign = Message::new([0; 32]);
    let signature1: B512 = wallet_1
        .signer()
        .sign(data_to_sign)
        .await?
        .as_ref()
        .try_into()?;
    let signature2: B512 = wallet_2
        .signer()
        .sign(data_to_sign)
        .await?
        .as_ref()
        .try_into()?;
    let signature3: B512 = wallet_3
        .signer()
        .sign(data_to_sign)
        .await?
        .as_ref()
        .try_into()?;

    let signatures = [signature1, signature2, signature3];

    let predicate_data = TestPredicateEncoder::default().encode_data(signatures)?;
    let code_path =
        "out/ec_recover_and_match_predicate.bin";

    let predicate = Predicate::load_from(code_path)?
        .with_data(predicate_data)
        .with_provider(provider.clone());

    let amount_to_predicate = 1000;
    let asset_id = AssetId::default();

    wallet_1
        .transfer(
            predicate.address(),
            amount_to_predicate,
            asset_id,
            TxPolicies::default(),
        )
        .await?;

    let predicate_balance = provider
        .get_asset_balance(&predicate.address(), &asset_id)
        .await?;
    assert_eq!(predicate_balance, amount_to_predicate as u128);

    predicate
        .transfer(
            receiver.address(),
            amount_to_predicate,
            asset_id,
            TxPolicies::default(),
        )
        .await?;

    let receiver_balance_after = receiver.get_asset_balance(&asset_id).await?;
    assert_eq!(amount_to_predicate as u128, receiver_balance_after);

    let predicate_balance = provider
        .get_asset_balance(&predicate.address(), &asset_id)
        .await?;
    assert_eq!(predicate_balance, 0);

    Ok(())
}
