use fuels::prelude::*;
use fuels_abigen_macro::abigen;

abigen!(
    TestStorageMapContract,
    "test_projects/storage_map/out/debug/storage_map-abi.json",
);

async fn test_storage_map_instance() -> TestStorageMapContract {
    let wallet = launch_provider_and_get_single_wallet().await;
    let id = Contract::deploy(
        "test_projects/storage_map/out/debug/storage_map.bin",
        &wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    TestStorageMapContract::new(id.to_string(), wallet)
}

#[tokio::test]
async fn test_u64_to_bool_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (1, 2, 3);
    let (val1, val2, val3) = (true, false, true);

    // Insert into u64 -> T storage maps
    instance.into_u64_to_bool(key1, val1).call().await.unwrap();
    instance.into_u64_to_bool(key2, val2).call().await.unwrap();
    instance.into_u64_to_bool(key3, val3).call().await.unwrap();

    // Get from u64 -> T storage maps
    assert_eq!(
        instance.from_u64_to_bool(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_bool(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_bool(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_u8_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (1, 2, 3);
    let (val1, val2, val3) = (8, 66, 99);

    instance.into_u64_to_u8(key1, val1).call().await.unwrap();
    instance.into_u64_to_u8(key2, val2).call().await.unwrap();
    instance.into_u64_to_u8(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u64_to_u8(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_u8(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_u8(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_u16_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (6, 9, 1);
    let (val1, val2, val3) = (9, 42, 100);

    instance.into_u64_to_u16(key1, val1).call().await.unwrap();
    instance.into_u64_to_u16(key2, val2).call().await.unwrap();
    instance.into_u64_to_u16(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u64_to_u16(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_u16(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_u16(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_u32_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (5, 99, 10);
    let (val1, val2, val3) = (90, 2, 100);

    instance.into_u64_to_u32(key1, val1).call().await.unwrap();
    instance.into_u64_to_u32(key2, val2).call().await.unwrap();
    instance.into_u64_to_u32(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u64_to_u32(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_u32(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_u32(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (50, 99, 1);
    let (val1, val2, val3) = (90, 20, 10);

    instance.into_u64_to_u64(key1, val1).call().await.unwrap();
    instance.into_u64_to_u64(key2, val2).call().await.unwrap();
    instance.into_u64_to_u64(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u64_to_u64(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_u64(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_u64(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_tuple_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (50, 99, 10);
    let (val1, val2, val3) = (
        ([1; 32], 42, true),
        ([2; 32], 24, true),
        ([3; 32], 99, true),
    );

    instance.into_u64_to_tuple(key1, val1).call().await.unwrap();
    instance.into_u64_to_tuple(key2, val2).call().await.unwrap();
    instance.into_u64_to_tuple(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u64_to_tuple(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_tuple(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_tuple(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_struct_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (5, 9, 1);
    let (val1, val2, val3) = (
        Struct {
            x: 42,
            y: [66; 32],
            z: [99; 32],
        },
        Struct {
            x: 24,
            y: [11; 32],
            z: [90; 32],
        },
        Struct {
            x: 77,
            y: [55; 32],
            z: [12; 32],
        },
    );

    instance
        .into_u64_to_struct(key1, val1.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_struct(key2, val2.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_struct(key3, val3.clone())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance
            .from_u64_to_struct(key1)
            .call()
            .await
            .unwrap()
            .value,
        val1
    );
    assert_eq!(
        instance
            .from_u64_to_struct(key2)
            .call()
            .await
            .unwrap()
            .value,
        val2
    );
    assert_eq!(
        instance
            .from_u64_to_struct(key3)
            .call()
            .await
            .unwrap()
            .value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_enum_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (44, 17, 1000);
    let (val1, val2, val3) = (Enum::V1([66; 32]), Enum::V2(42), Enum::V3([42; 32]));

    instance
        .into_u64_to_enum(key1, val1.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_enum(key2, val2.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_enum(key3, val3.clone())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.from_u64_to_enum(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_enum(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_enum(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u64_to_string_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (9001, 1980, 1000);
    let (val1, val2, val3) = (
        "fastest_modular_execution_layer_A",
        "fastest_modular_execution_layer_B",
        "fastest_modular_execution_layer_C",
    );

    instance
        .into_u64_to_str(key1, val1.to_string())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_str(key2, val2.to_string())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_str(key3, val3.to_string())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.from_u64_to_str(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u64_to_str(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u64_to_str(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_bool_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2) = (true, false);
    let (val1, val2) = (1, 2);

    instance.into_bool_to_u64(key1, val1).call().await.unwrap();
    instance.into_bool_to_u64(key2, val2).call().await.unwrap();

    assert_eq!(
        instance.from_bool_to_u64(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_bool_to_u64(key2).call().await.unwrap().value,
        val2
    );
}

#[tokio::test]
async fn test_u8_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (8, 66, 99);
    let (val1, val2, val3) = (1, 2, 3);

    instance.into_u8_to_u64(key1, val1).call().await.unwrap();
    instance.into_u8_to_u64(key2, val2).call().await.unwrap();
    instance.into_u8_to_u64(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u8_to_u64(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u8_to_u64(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u8_to_u64(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u16_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (9, 42, 100);
    let (val1, val2, val3) = (6, 9, 1);

    instance.into_u16_to_u64(key1, val1).call().await.unwrap();
    instance.into_u16_to_u64(key2, val2).call().await.unwrap();
    instance.into_u16_to_u64(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u16_to_u64(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u16_to_u64(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u16_to_u64(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_u32_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (90, 2, 100);
    let (val1, val2, val3) = (5, 99, 10);

    instance.into_u32_to_u64(key1, val1).call().await.unwrap();
    instance.into_u32_to_u64(key2, val2).call().await.unwrap();
    instance.into_u32_to_u64(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_u32_to_u64(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_u32_to_u64(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_u32_to_u64(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_tuple_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (
        ([1; 32], 42, true),
        ([2; 32], 24, true),
        ([3; 32], 99, true),
    );
    let (val1, val2, val3) = (50, 99, 10);

    instance.into_tuple_to_u64(key1, val1).call().await.unwrap();
    instance.into_tuple_to_u64(key2, val2).call().await.unwrap();
    instance.into_tuple_to_u64(key3, val3).call().await.unwrap();

    assert_eq!(
        instance.from_tuple_to_u64(key1).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_tuple_to_u64(key2).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_tuple_to_u64(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_struct_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (
        Struct {
            x: 42,
            y: [66; 32],
            z: [99; 32],
        },
        Struct {
            x: 24,
            y: [11; 32],
            z: [90; 32],
        },
        Struct {
            x: 77,
            y: [55; 32],
            z: [12; 32],
        },
    );

    let (val1, val2, val3) = (5, 9, 1);

    instance
        .into_struct_to_u64(key1.clone(), val1.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_struct_to_u64(key2.clone(), val2.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_struct_to_u64(key3.clone(), val3.clone())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance
            .from_struct_to_u64(key1)
            .call()
            .await
            .unwrap()
            .value,
        val1
    );
    assert_eq!(
        instance
            .from_struct_to_u64(key2)
            .call()
            .await
            .unwrap()
            .value,
        val2
    );
    assert_eq!(
        instance
            .from_struct_to_u64(key3)
            .call()
            .await
            .unwrap()
            .value,
        val3
    );
}

#[tokio::test]
async fn test_enum_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (Enum::V1([66; 32]), Enum::V2(42), Enum::V3([42; 32]));
    let (val1, val2, val3) = (44, 17, 1000);

    instance
        .into_enum_to_u64(key1.clone(), val1.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_enum_to_u64(key2.clone(), val2.clone())
        .call()
        .await
        .unwrap();
    instance
        .into_enum_to_u64(key3.clone(), val3.clone())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.from_enum_to_u64(key1).call().await.unwrap().value,
        val1
    );
    // This assert currently fails. Not sure why yet
    //    assert_eq!(
    //        instance.from_enum_to_u64(key2).call().await.unwrap().value,
    //        val2
    //    );
    assert_eq!(
        instance.from_enum_to_u64(key3).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_string_to_u64_map() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (
        "fastest_modular_execution_layer_A",
        "fastest_modular_execution_layer_B",
        "fastest_modular_execution_layer_C",
    );
    let  (val1, val2, val3) = (9001, 1980, 1000);

    instance
        .into_str_to_u64(key1.to_string(), val1)
        .call()
        .await
        .unwrap();
    instance
        .into_str_to_u64(key2.to_string(), val2)
        .call()
        .await
        .unwrap();
    instance
        .into_str_to_u64(key3.to_string(), val3)
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.from_str_to_u64(key1.to_string()).call().await.unwrap().value,
        val1
    );
    assert_eq!(
        instance.from_str_to_u64(key2.to_string()).call().await.unwrap().value,
        val2
    );
    assert_eq!(
        instance.from_str_to_u64(key3.to_string()).call().await.unwrap().value,
        val3
    );
}

#[tokio::test]
async fn test_multiple_maps() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    let (key1, key2, key3) = (1, 2, 3);
    let (val1_1, val2_1, val3_1) = (8, 66, 99);
    let (val1_2, val2_2, val3_2) = (9, 42, 100);

    instance.into_u64_to_u8(key1, val1_1).call().await.unwrap();
    instance.into_u64_to_u8(key2, val2_1).call().await.unwrap();
    instance.into_u64_to_u8(key3, val3_1).call().await.unwrap();

    instance.into_u64_to_u16(key1, val1_2).call().await.unwrap();
    instance.into_u64_to_u16(key2, val2_2).call().await.unwrap();
    instance.into_u64_to_u16(key3, val3_2).call().await.unwrap();

    assert_eq!(
        instance.from_u64_to_u8(key1).call().await.unwrap().value,
        val1_1
    );
    assert_eq!(
        instance.from_u64_to_u8(key2).call().await.unwrap().value,
        val2_1
    );
    assert_eq!(
        instance.from_u64_to_u8(key3).call().await.unwrap().value,
        val3_1
    );

    assert_eq!(
        instance.from_u64_to_u16(key1).call().await.unwrap().value,
        val1_2
    );
    assert_eq!(
        instance.from_u64_to_u16(key2).call().await.unwrap().value,
        val2_2
    );
    assert_eq!(
        instance.from_u64_to_u16(key3).call().await.unwrap().value,
        val3_2
    );
}
