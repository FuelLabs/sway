use fuels::{accounts::wallet::Wallet, prelude::*};

abigen!(Contract(
    name = "ExpectTestingContract",
    abi = "test_projects/result_option_expect/out/release/result_option_expect-abi.json"
));

async fn setup() -> (ExpectTestingContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/result_option_expect/out/release/result_option_expect.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = ExpectTestingContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_expect_option() {
    let (instance, _id) = setup().await;

    instance
        .methods()
        .option_test_should_not_revert()
        .call()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_expect_result() {
    let (instance, _id) = setup().await;

    instance
        .methods()
        .result_test_should_not_revert()
        .call()
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic]
async fn test_expect_option_panic() {
    let (instance, _id) = setup().await;

    instance
        .methods()
        .option_test_should_revert()
        .call()
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic]
async fn test_expect_result_panic() {
    let (instance, _id) = setup().await;

    instance
        .methods()
        .result_test_should_revert()
        .call()
        .await
        .unwrap();
}
