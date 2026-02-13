use fuels::{
    prelude::*,
    types::{Bits256, SizedAsciiString},
};

abigen!(Contract(
    name = "TestStorageContract",
    abi = "out_for_sdk_harness_tests/storage-abi.json",
));

async fn get_test_storage_instance() -> TestStorageContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out_for_sdk_harness_tests/storage.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestStorageContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn can_store_and_get_bool() {
    let instance = get_test_storage_instance().await;
    let b = true;

    // Test store
    instance.methods().store_bool(b).call().await.unwrap();
    let result = instance.methods().get_bool().call().await.unwrap();
    assert_eq!(result.value, Some(b));
}

#[tokio::test]
async fn can_store_and_get_u8() {
    let instance = get_test_storage_instance().await;
    let n = 8;

    // Test store
    instance.methods().store_u8(n).call().await.unwrap();
    let result = instance.methods().get_u8().call().await.unwrap();
    assert_eq!(result.value, Some(n));
}

#[tokio::test]
async fn can_store_and_get_u16() {
    let instance = get_test_storage_instance().await;
    let n = 16;

    // Test store
    instance.methods().store_u16(n).call().await.unwrap();
    let result = instance.methods().get_u16().call().await.unwrap();
    assert_eq!(result.value, Some(n));
}

#[tokio::test]
async fn can_store_and_get_u32() {
    let instance = get_test_storage_instance().await;
    let n = 32;

    // Test store
    instance.methods().store_u32(n).call().await.unwrap();
    let result = instance.methods().get_u32().call().await.unwrap();
    assert_eq!(result.value, Some(n));
}

#[tokio::test]
async fn can_store_and_get_u64() {
    let instance = get_test_storage_instance().await;
    let n = 64;

    // Test store
    instance.methods().store_u64(n).call().await.unwrap();
    let result = instance.methods().get_u64().call().await.unwrap();
    assert_eq!(result.value, Some(n));
}

#[tokio::test]
async fn can_store_b256() {
    let instance = get_test_storage_instance().await;
    let n: Bits256 = Bits256([2; 32]);

    // Test store
    instance.methods().store_b256(n).call().await.unwrap();
    let result = instance.methods().get_b256().call().await.unwrap();
    assert_eq!(result.value, Some(n));
}

#[tokio::test]
async fn can_store_small_struct() {
    let instance = get_test_storage_instance().await;
    let s = SmallStruct { x: 42 };

    // Test store
    instance
        .methods()
        .store_small_struct(s.clone())
        .call()
        .await
        .unwrap();
    let result = instance.methods().get_small_struct().call().await.unwrap();
    assert_eq!(result.value, Some(s));
}

#[tokio::test]
async fn can_store_medium_struct() {
    let instance = get_test_storage_instance().await;
    let s = MediumStruct { x: 42, y: 66 };

    // Test store
    instance
        .methods()
        .store_medium_struct(s.clone())
        .call()
        .await
        .unwrap();
    let result = instance.methods().get_medium_struct().call().await.unwrap();
    assert_eq!(result.value, Some(s));
}

#[tokio::test]
async fn can_store_large_struct() {
    let instance = get_test_storage_instance().await;
    let s = LargeStruct {
        x: 13,
        y: Bits256([6; 32]),
        z: 77,
    };

    // Test store
    instance
        .methods()
        .store_large_struct(s.clone())
        .call()
        .await
        .unwrap();
    let result = instance.methods().get_large_struct().call().await.unwrap();
    assert_eq!(result.value, Some(s));
}

#[tokio::test]
async fn can_store_very_large_struct() {
    let instance = get_test_storage_instance().await;
    let s = VeryLargeStruct {
        x: 42,
        y: Bits256([9; 32]),
        z: Bits256([7; 32]),
    };
    instance
        .methods()
        .store_very_large_struct(s.clone())
        .call()
        .await
        .unwrap();
    let result = instance
        .methods()
        .get_very_large_struct()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, Some(s));
}

#[tokio::test]
async fn can_store_enum() {
    let instance = get_test_storage_instance().await;
    let e1 = StorageEnum::V1(Bits256([3; 32]));

    // Test store
    instance
        .methods()
        .store_enum(e1.clone())
        .call()
        .await
        .unwrap();
    let result = instance.methods().get_enum().call().await.unwrap();
    assert_eq!(result.value, Some(e1));

    let e2 = StorageEnum::V2(99);
    instance
        .methods()
        .store_enum(e2.clone())
        .call()
        .await
        .unwrap();
    let result = instance.methods().get_enum().call().await.unwrap();
    assert_eq!(result.value, Some(e2));

    let e3 = StorageEnum::V3(Bits256([4; 32]));
    instance
        .methods()
        .store_enum(e3.clone())
        .call()
        .await
        .unwrap();
    let result = instance.methods().get_enum().call().await.unwrap();
    assert_eq!(result.value, Some(e3));
}

#[tokio::test]
async fn can_store_tuple() {
    let instance = get_test_storage_instance().await;
    let t = (Bits256([7; 32]), 8, Bits256([6; 32]));

    // Test store
    instance.methods().store_tuple(t).call().await.unwrap();
    let result = instance.methods().get_tuple().call().await.unwrap();
    assert_eq!(result.value, Some(t));
}

#[tokio::test]
async fn can_store_string() {
    let instance = get_test_storage_instance().await;
    let s = "fastest_modular_execution_layer".to_string();

    // Test store
    instance
        .methods()
        .store_string(SizedAsciiString::try_from(s.clone()).unwrap())
        .call()
        .await
        .unwrap();
    let result = instance.methods().get_string().call().await.unwrap();
    assert_eq!(result.value, Some(SizedAsciiString::try_from(s).unwrap()));
}

#[tokio::test]
async fn can_store_array() {
    let instance = get_test_storage_instance().await;
    let a = [Bits256([153; 32]), Bits256([136; 32]), Bits256([119; 32])];

    // Test store
    instance.methods().store_array().call().await.unwrap();
    let result = instance.methods().get_array().call().await.unwrap();
    assert_eq!(result.value, Some(a));
}

#[tokio::test]
async fn can_store_non_inlined() {
    let instance = get_test_storage_instance().await;
    let result = instance.methods().storage_in_call().call().await.unwrap();
    assert_eq!(result.value, 333);
}
