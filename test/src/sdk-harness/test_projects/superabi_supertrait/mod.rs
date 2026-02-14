use fuels::prelude::*;

abigen!(Contract(
    name = "SuperAbiSuperTraitTestContract",
    abi = "out/superabi_supertrait-abi.json"
));

async fn get_superabi_supertrait_instance() -> SuperAbiSuperTraitTestContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out/superabi_supertrait.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    SuperAbiSuperTraitTestContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn method1_test() -> Result<()> {
    let instance = get_superabi_supertrait_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.method1().call().await?;
    assert_eq!(42, response.value);

    Ok(())
}

// contract supertrait methods are not callable externally
// #[tokio::test]
// async fn method_test() -> Result<()> {
//     let instance = get_superabi_supertrait_instance().await;
//     let contract_methods = instance.methods();

//     let response = contract_methods.method().call().await?;
//     assert_eq!(0xBAD, response.value);

//     Ok(())
// }
