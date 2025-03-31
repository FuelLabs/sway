use fuels::{
    accounts::{predicate::Predicate, wallet::Wallet, Account},
    crypto::Message,
    prelude::*,
    types::B512,
};
use crate::new_random_wallet;

abigen!(
    Predicate(
        name = "TestPredicate",
        abi = "test_projects/ec_recover_and_match_predicate/out/release/ec_recover_and_match_predicate-abi.json"
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

    let mut receiver = new_random_wallet(None);

    let signer = PrivateKeySigner::new(secret_key1);
    let signer2 = PrivateKeySigner::new(secret_key2);
    let signer3 = PrivateKeySigner::new(secret_key3);

    let all_coins = [&signer, &signer2, &signer3]
        .iter()
        .flat_map(|signer| {
            setup_single_asset_coins(signer.address(), AssetId::default(), 10, 1_000_000)
        })
        .collect::<Vec<_>>();

    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let provider = setup_test_provider(all_coins, vec![], Some(node_config), None)
        .await
        .unwrap();

    let mut wallet = Wallet::new(signer, provider.clone());
    let mut wallet2 = Wallet::new(signer2, provider.clone());
    let mut wallet3 = Wallet::new(signer3, provider.clone());

    let data_to_sign = Message::new([0; 32]);
    let signature1: B512 = wallet.sign(data_to_sign).await?.as_ref().try_into()?;
    let signature2: B512 = wallet2.sign(data_to_sign).await?.as_ref().try_into()?;
    let signature3: B512 = wallet3.sign(data_to_sign).await?.as_ref().try_into()?;

    let signatures = [signature1, signature2, signature3];

    let predicate_data = TestPredicateEncoder::default().encode_data(signatures)?;
    let code_path =
        "test_projects/ec_recover_and_match_predicate/out/release/ec_recover_and_match_predicate.bin";

    let predicate = Predicate::load_from(code_path)?
        .with_data(predicate_data)
        .with_provider(provider.clone());

    let amount_to_predicate = 1000;
    let asset_id = AssetId::default();

    wallet
        .transfer(
            predicate.address(),
            amount_to_predicate,
            asset_id,
            TxPolicies::default(),
        )
        .await?;

    let predicate_balance = provider
        .get_asset_balance(predicate.address(), asset_id)
        .await?;
    assert_eq!(predicate_balance, amount_to_predicate);

    predicate
        .transfer(
            receiver.address(),
            amount_to_predicate,
            asset_id,
            TxPolicies::default(),
        )
        .await?;

    let receiver_balance_after = receiver.get_asset_balance(&asset_id).await?;
    assert_eq!(amount_to_predicate, receiver_balance_after);

    let predicate_balance = provider
        .get_asset_balance(predicate.address(), asset_id)
        .await?;
    assert_eq!(predicate_balance, 0);

    Ok(())
}
