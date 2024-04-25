use fuels::{accounts::wallet::WalletUnlocked, prelude::*, types::AssetId};

abigen!(Contract(
    name = "TestAssetId",
    abi = "test_projects/asset_id/out/release/asset_id-abi.json"
));

#[tokio::test]
async fn can_get_base_asset_id() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, _fuelcontract_id) = get_instance(wallet.clone()).await;

    let asset_id = fuelcontract_instance
        .methods()
        .get_base_asset_id()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(asset_id, *wallet.provider().unwrap().base_asset_id());
}

async fn get_instance(wallet: WalletUnlocked) -> (TestAssetId<WalletUnlocked>, ContractId) {
    let fuelcontract_id = Contract::load_from(
        "test_projects/asset_id/out/release/asset_id.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    wallet
        .force_transfer_to_contract(&fuelcontract_id, 1000, AssetId::BASE, TxPolicies::default())
        .await
        .unwrap();
    let fuelcontract_instance = TestAssetId::new(fuelcontract_id.clone(), wallet);

    (fuelcontract_instance, fuelcontract_id.into())
}
