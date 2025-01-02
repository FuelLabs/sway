use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::*;

// TODO: Remove these tests and replace them with in-language tests.
abigen!(Contract(
    name = "TestStorageVecIterContract",
    abi = "test_projects/storage_vec_iter/out/release/storage_vec_iter-abi.json",
));

async fn setup() -> TestStorageVecIterContract<WalletUnlocked> {
    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(1), None, None),
        Some(node_config),
        None,
    )
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();
    let id = Contract::load_from(
        "test_projects/storage_vec_iter/out/release/storage_vec_iter.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    TestStorageVecIterContract::new(id, wallet)
}

#[tokio::test]
async fn for_u64() {
    let instance = setup().await;

    let mut input = Vec::new();
    for i in 0..100 {
        input.push(i);
    }

    let _ = instance.methods().store(input.clone()).call().await;

    assert!(
        instance
            .methods()
            .for_iter(input)
            .call()
            .await
            .unwrap()
            .value
    );
}

#[tokio::test]
async fn next_u64() {
    let instance = setup().await;

    let mut input = Vec::new();
    for i in 0..100 {
        input.push(i);
    }

    let _ = instance.methods().store(input.clone()).call().await;

    assert!(
        instance
            .methods()
            .next_iter(input)
            .call()
            .await
            .unwrap()
            .value
    );
}
