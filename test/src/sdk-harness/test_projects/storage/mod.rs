use fuels::prelude::*;

abigen!(
    TestStorageContract,
    "test_projects/storage/out/debug/storage-abi.json",
);

async fn get_test_storage_instance() -> TestStorageContract {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/storage/out/debug/storage.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/storage/out/debug/storage-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    TestStorageContractBuilder::new(id.to_string(), wallet).build()
}

#[tokio::test]
async fn can_store_and_get_bool() {
    let instance = get_test_storage_instance().await;
    let b = true;
    instance.store_bool(b).call().await.unwrap();
    let result = instance.get_bool().call().await.unwrap();
    assert_eq!(result.value, b);
}

#[tokio::test]
async fn can_store_and_get_u8() {
    let instance = get_test_storage_instance().await;
    let n = 8;
    instance.store_u8(n).call().await.unwrap();
    let result = instance.get_u8().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_and_get_u16() {
    let instance = get_test_storage_instance().await;
    let n = 16;
    instance.store_u16(n).call().await.unwrap();
    let result = instance.get_u16().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_and_get_u32() {
    let instance = get_test_storage_instance().await;
    let n = 32;
    instance.store_u32(n).call().await.unwrap();
    let result = instance.get_u32().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_and_get_u64() {
    let instance = get_test_storage_instance().await;
    let n = 64;
    instance.store_u64(n).call().await.unwrap();
    let result = instance.get_u64().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_b256() {
    let instance = get_test_storage_instance().await;
    let n: [u8; 32] = [2; 32];
    instance.store_b256(n).call().await.unwrap();
    let result = instance.get_b256().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_small_struct() {
    let instance = get_test_storage_instance().await;
    let s = SmallStruct { x: 42 };
    instance.store_small_struct(s.clone()).call().await.unwrap();
    let result = instance.get_small_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_medium_struct() {
    let instance = get_test_storage_instance().await;
    let s = MediumStruct { x: 42, y: 66 };
    instance
        .store_medium_struct(s.clone())
        .call()
        .await
        .unwrap();
    let result = instance.get_medium_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_large_struct() {
    let instance = get_test_storage_instance().await;
    let s = LargeStruct {
        x: 13,
        y: [6; 32],
        z: 77,
    };
    instance.store_large_struct(s.clone()).call().await.unwrap();
    let result = instance.get_large_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_very_large_struct() {
    let instance = get_test_storage_instance().await;
    let s = VeryLargeStruct {
        x: 42,
        y: [9; 32],
        z: [7; 32],
    };
    instance
        .store_very_large_struct(s.clone())
        .call()
        .await
        .unwrap();
    let result = instance.get_very_large_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_enum() {
    let instance = get_test_storage_instance().await;
    let e1 = StorageEnum::V1([3; 32]);
    instance.store_enum(e1.clone()).call().await.unwrap();
    let result = instance.get_enum().call().await.unwrap();
    assert_eq!(result.value, e1);

    let e2 = StorageEnum::V2(99);
    instance.store_enum(e2.clone()).call().await.unwrap();
    let result = instance.get_enum().call().await.unwrap();
    assert_eq!(result.value, e2);

    let e3 = StorageEnum::V3([4; 32]);
    instance.store_enum(e3.clone()).call().await.unwrap();
    let result = instance.get_enum().call().await.unwrap();
    assert_eq!(result.value, e3);
}

#[tokio::test]
async fn can_store_tuple() {
    let instance = get_test_storage_instance().await;
    let t = ([7; 32], 8, [6; 32]);
    instance.store_tuple(t.clone()).call().await.unwrap();
    let result = instance.get_tuple().call().await.unwrap();
    assert_eq!(result.value, t);
}

#[tokio::test]
async fn can_store_string() {
    let instance = get_test_storage_instance().await;
    let s = "fastest_modular_execution_layer".to_string();
    instance.store_string(s.clone()).call().await.unwrap();
    let result = instance.get_string().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_array() {
    let instance = get_test_storage_instance().await;
    let a = [[153; 32], [136; 32], [119; 32]].to_vec();
    instance.store_array().call().await.unwrap();
    let result = instance.get_array().call().await.unwrap();
    assert_eq!(result.value, a);
}
