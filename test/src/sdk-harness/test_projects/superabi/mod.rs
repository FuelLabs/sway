use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::*;

abigen!(Contract(
    name = "SuperAbiTestContract",
    abi = "test_projects/superabi/out/release/superabi-abi.json"
));

async fn get_superabi_instance() -> SuperAbiTestContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/superabi/out/release/superabi.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();
    SuperAbiTestContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn abi_test() -> Result<()> {
    let instance = get_superabi_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.abi_test().call().await?;
    assert_eq!(42, response.value);

    Ok(())
}

#[tokio::test]
async fn superabi_test() -> Result<()> {
    let instance = get_superabi_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.superabi_test().call().await?;
    assert_eq!(41, response.value);

    Ok(())
}
