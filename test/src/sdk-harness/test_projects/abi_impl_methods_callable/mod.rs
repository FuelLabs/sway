use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::*;

abigen!(Contract(
    name = "AbiImplMethodsCallable",
    abi = "test_projects/abi_impl_methods_callable/out/release/abi_impl_methods_callable-abi.json"
));

async fn get_abi_impl_methods_callable_instance() -> AbiImplMethodsCallable<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/abi_impl_methods_callable/out/release/abi_impl_methods_callable.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();
    AbiImplMethodsCallable::new(id.clone(), wallet)
}

#[tokio::test]
async fn impl_method_test() -> Result<()> {
    let instance = get_abi_impl_methods_callable_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.impl_method().call().await?;
    assert_eq!(42, response.value);

    Ok(())
}
