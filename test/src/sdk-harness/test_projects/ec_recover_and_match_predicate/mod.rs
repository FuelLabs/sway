use fuels::prelude::*;

#[tokio::test]
async fn ec_recover_and_match_predicate_test() -> Result<(), Error> {
    use fuels::contract::predicate::Predicate;
    use fuels::signers::fuel_crypto::SecretKey;

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

    let mut wallet = WalletUnlocked::new_from_private_key(secret_key1, None);
    let mut wallet2 = WalletUnlocked::new_from_private_key(secret_key2, None);
    let mut wallet3 = WalletUnlocked::new_from_private_key(secret_key3, None);
    let receiver = WalletUnlocked::new_random(None);

    let all_coins = [&wallet, &wallet2, &wallet3]
        .iter()
        .flat_map(|wallet| {
            setup_single_asset_coins(wallet.address(), AssetId::default(), 10, 1_000_000)
        })
        .collect::<Vec<_>>();

    let (provider, _) = setup_test_provider(
        all_coins,
        vec![],
        Some(Config {
            utxo_validation: true,
            ..Config::local_node()
        }),
    )
    .await;

    [&mut wallet, &mut wallet2, &mut wallet3]
        .iter_mut()
        .for_each(|wallet| wallet.set_provider(provider.clone()));

    let predicate = Predicate::load_from(
        "test_projects/ec_recover_and_match_predicate/out/debug/ec_recover_and_match_predicate.bin",
    )?;

    let predicate_code = predicate.code();
    let predicate_address = predicate.address();
    let amount_to_predicate = 1000;
    let asset_id = AssetId::default();

    wallet
        .transfer(
            predicate_address,
            amount_to_predicate,
            asset_id,
            TxParameters::default(),
        )
        .await?;

    let predicate_balance = provider
        .get_asset_balance(predicate.address(), asset_id)
        .await?;
    assert_eq!(predicate_balance, amount_to_predicate);

    let data_to_sign = [0; 32];
    let signature1 = wallet.sign_message(&data_to_sign).await?.to_vec();
    let signature2 = wallet2.sign_message(&data_to_sign).await?.to_vec();
    let signature3 = wallet3.sign_message(&data_to_sign).await?.to_vec();

    let signatures = vec![signature1, signature2, signature3];

    let predicate_data = signatures.into_iter().flatten().collect();
    wallet
        .spend_predicate(
            predicate_address,
            predicate_code,
            amount_to_predicate,
            asset_id,
            receiver.address(),
            Some(predicate_data),
            TxParameters::default(),
        )
        .await?;

    let receiver_balance_after = provider
        .get_asset_balance(receiver.address(), asset_id)
        .await?;
    assert_eq!(amount_to_predicate, receiver_balance_after);

    let predicate_balance = provider
        .get_asset_balance(predicate.address(), asset_id)
        .await?;
    assert_eq!(predicate_balance, 0);

    Ok(())
}
