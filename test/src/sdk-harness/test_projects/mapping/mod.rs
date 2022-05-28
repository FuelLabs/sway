use fuels::prelude::*;
use fuels_abigen_macro::abigen;

abigen!(
    TestMappingContract,
    "test_projects/mapping/out/debug/mapping-abi.json",
);

async fn get_test_mapping_instance() -> TestMappingContract {
    let wallet = launch_provider_and_get_single_wallet().await;
    let id = Contract::deploy(
        "test_projects/mapping/out/debug/mapping.bin",
        &wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    TestMappingContract::new(id.to_string(), wallet)
}

#[tokio::test]
async fn can_store_and_get_bool() {
    let instance = get_test_mapping_instance().await;

    instance.init().call().await;

    instance.insert_into_mapping1(1, 42).call().await;
    instance.insert_into_mapping1(2, 77).call().await;
    instance.insert_into_mapping2([1; 32], true).call().await;
    instance.insert_into_mapping2([2; 32], true).call().await;
    instance.insert_into_mapping3((42, true), [7; 32]).call().await;
    instance.insert_into_mapping3((99, false), [8; 32]).call().await;

    assert_eq!(instance.get_from_mapping1(1).call().await.unwrap().value, 42);
    assert_eq!(instance.get_from_mapping1(2).call().await.unwrap().value, 77);
    assert_eq!(instance.get_from_mapping2([1; 32]).call().await.unwrap().value, true);
    assert_eq!(instance.get_from_mapping2([2; 32]).call().await.unwrap().value, true);
    assert_eq!(instance.get_from_mapping3((42, true)).call().await.unwrap().value, [7; 32]);
    assert_eq!(instance.get_from_mapping3((99, false)).call().await.unwrap().value, [8; 32]);


}
