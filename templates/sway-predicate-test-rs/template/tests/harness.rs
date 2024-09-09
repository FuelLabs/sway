use fuels::{accounts::{predicate::Predicate}, prelude::*};

// Load abi from json
abigen!(Predicate(
    name = "MyPredicate",
    abi = "out/debug/{{project-name}}-abi.json"
));

async fn get_predicate_instance() -> (WalletUnlocked, Predicate, AssetId) {
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    let wallet = wallets.pop().unwrap();

    let provider = wallet.provider().clone().unwrap();

    let base_asset_id = provider.base_asset_id().clone();

    let bin_path = "./out/debug/{{project-name}}.bin";

    let instance: Predicate = Predicate::load_from(bin_path).unwrap().with_provider(provider.clone());

    (wallet, instance, base_asset_id)
}

async fn check_balances(
    wallet: &WalletUnlocked, 
    instance: &Predicate, 
    expected_wallet_balance: Option<u64>, 
    expected_predicate_balance: Option<u64>,
) -> (u64, u64) {
    let wallet_bal = wallet.get_asset_balance(&AssetId::default()).await.unwrap();
    let predicate_bal = instance.get_asset_balance(&AssetId::default()).await.unwrap();

    if let Some(expected) = expected_wallet_balance {
        assert_eq!(wallet_bal, expected);
    }

    if let Some(expected) = expected_predicate_balance {
        assert_eq!(predicate_bal, expected);
    }

    (wallet_bal, predicate_bal)
}

#[tokio::test]
async fn can_get_predicate_instance() {
    let (wallet, instance, base_asset_id) = get_predicate_instance().await;
    let predicate_root = instance.address();

    // Check balances before funding predicate
    check_balances(&wallet, &instance, Some(1_000_000_000u64), None).await;

    // Fund predicate from wallet
    let _ = wallet.transfer(predicate_root, 1234, base_asset_id, TxPolicies::default()).await;

    // Check balances after funding predicate
    check_balances(&wallet, &instance, Some(999_998_766u64), Some(1234u64)).await;

    let _ = instance.transfer(wallet.address(), 1234, base_asset_id, TxPolicies::default()).await;

    // Check balances after transferring funds out of predicate
    check_balances(&wallet, &instance, Some(1_000_000_000u64), Some(0u64)).await;
}
