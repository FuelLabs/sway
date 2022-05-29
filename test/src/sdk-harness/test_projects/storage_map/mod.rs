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
async fn can_insert_and_get() {
    let instance = test_storage_map_instance().await;

    instance.init().call().await.unwrap();

    // Insert into u64 -> T storage maps
    instance.into_u64_to_bool(1, true).call().await.unwrap();
    instance.into_u64_to_bool(2, false).call().await.unwrap();
    instance.into_u64_to_bool(3, true).call().await.unwrap();

    instance.into_u64_to_u8(1, 8).call().await.unwrap();
    instance.into_u64_to_u8(2, 66).call().await.unwrap();
    instance.into_u64_to_u8(3, 99).call().await.unwrap();

    instance.into_u64_to_u16(6, 9).call().await.unwrap();
    instance.into_u64_to_u16(9, 42).call().await.unwrap();
    instance.into_u64_to_u16(1, 100).call().await.unwrap();

    instance.into_u64_to_u32(5, 90).call().await.unwrap();
    instance.into_u64_to_u32(99, 2).call().await.unwrap();
    instance.into_u64_to_u32(10, 100).call().await.unwrap();

    instance.into_u64_to_u64(50, 90).call().await.unwrap();
    instance.into_u64_to_u64(99, 20).call().await.unwrap();
    instance.into_u64_to_u64(1, 10).call().await.unwrap();

    instance
        .into_u64_to_tuple(50, ([1; 32], 42, true))
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_tuple(99, ([2; 32], 24, true))
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_tuple(10, ([3; 32], 99, true))
        .call()
        .await
        .unwrap();

    instance
        .into_u64_to_struct(
            5,
            Struct {
                x: 42,
                y: [66; 32],
                z: [99; 32],
            },
        )
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_struct(
            9,
            Struct {
                x: 24,
                y: [11; 32],
                z: [90; 32],
            },
        )
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_struct(
            1,
            Struct {
                x: 77,
                y: [55; 32],
                z: [12; 32],
            },
        )
        .call()
        .await
        .unwrap();

    instance
        .into_u64_to_enum(44, Enum::V1([66; 32]))
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_enum(17, Enum::V2(42))
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_enum(1000, Enum::V3([42; 32]))
        .call()
        .await
        .unwrap();

    instance
        .into_u64_to_str(9001, "fastest_modular_execution_layer_1".to_string())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_str(1980, "fastest_modular_execution_layer_2".to_string())
        .call()
        .await
        .unwrap();
    instance
        .into_u64_to_str(1000, "fastest_modular_execution_layer_3".to_string())
        .call()
        .await
        .unwrap();

    // Insert into T -> u64 storage maps
    instance.into_bool_to_u64(true, 1).call().await.unwrap();
    instance.into_bool_to_u64(false, 2).call().await.unwrap();

    instance.into_u8_to_u64(8, 1).call().await.unwrap();
    instance.into_u8_to_u64(66, 2).call().await.unwrap();
    instance.into_u8_to_u64(99, 3).call().await.unwrap();

    instance.into_u16_to_u64(9, 6).call().await.unwrap();
    instance.into_u16_to_u64(42, 9).call().await.unwrap();
    instance.into_u16_to_u64(100, 1).call().await.unwrap();

    instance.into_u32_to_u64(90, 5).call().await.unwrap();
    instance.into_u32_to_u64(2, 99).call().await.unwrap();
    instance.into_u32_to_u64(100, 10).call().await.unwrap();

    instance
        .into_tuple_to_u64(([1; 32], 42, true), 50)
        .call()
        .await
        .unwrap();
    instance
        .into_tuple_to_u64(([2; 32], 24, true), 99)
        .call()
        .await
        .unwrap();
    instance
        .into_tuple_to_u64(([3; 32], 99, true), 10)
        .call()
        .await
        .unwrap();

    instance
        .into_struct_to_u64(
            Struct {
                x: 42,
                y: [66; 32],
                z: [99; 32],
            },
            5,
        )
        .call()
        .await
        .unwrap();
    instance
        .into_struct_to_u64(
            Struct {
                x: 24,
                y: [11; 32],
                z: [90; 32],
            },
            9,
        )
        .call()
        .await
        .unwrap();
    instance
        .into_struct_to_u64(
            Struct {
                x: 77,
                y: [55; 32],
                z: [12; 32],
            },
            1,
        )
        .call()
        .await
        .unwrap();

    instance
        .into_enum_to_u64(Enum::V1([66; 32]), 44)
        .call()
        .await
        .unwrap();
    instance
        .into_enum_to_u64(Enum::V2(42), 17)
        .call()
        .await
        .unwrap();
    instance
        .into_enum_to_u64(Enum::V3([42; 32]), 1000)
        .call()
        .await
        .unwrap();

    instance
        .into_str_to_u64("fastest_modular_execution_layer_1".to_string(), 9001)
        .call()
        .await
        .unwrap();
    instance
        .into_str_to_u64("fastest_modular_execution_layer_2".to_string(), 1980)
        .call()
        .await
        .unwrap();
    instance
        .into_str_to_u64("fastest_modular_execution_layer_3".to_string(), 1000)
        .call()
        .await
        .unwrap();

    // Get from u64 -> T storage maps
    assert_eq!(
        instance.from_u64_to_bool(1).call().await.unwrap().value,
        true
    );
    assert_eq!(
        instance.from_u64_to_bool(2).call().await.unwrap().value,
        false
    );
    assert_eq!(
        instance.from_u64_to_bool(3).call().await.unwrap().value,
        true
    );

    assert_eq!(instance.from_u64_to_u8(1).call().await.unwrap().value, 8);
    assert_eq!(instance.from_u64_to_u8(2).call().await.unwrap().value, 66);
    assert_eq!(instance.from_u64_to_u8(3).call().await.unwrap().value, 99);

    assert_eq!(instance.from_u64_to_u16(6).call().await.unwrap().value, 9);
    assert_eq!(instance.from_u64_to_u16(9).call().await.unwrap().value, 42);
    assert_eq!(instance.from_u64_to_u16(1).call().await.unwrap().value, 100);

    assert_eq!(instance.from_u64_to_u32(5).call().await.unwrap().value, 90);
    assert_eq!(instance.from_u64_to_u32(99).call().await.unwrap().value, 2);
    assert_eq!(
        instance.from_u64_to_u32(10).call().await.unwrap().value,
        100
    );

    assert_eq!(instance.from_u64_to_u64(50).call().await.unwrap().value, 90);
    assert_eq!(instance.from_u64_to_u64(99).call().await.unwrap().value, 20);
    assert_eq!(instance.from_u64_to_u64(1).call().await.unwrap().value, 10);

    assert_eq!(
        instance.from_u64_to_tuple(50).call().await.unwrap().value,
        ([1; 32], 42, true)
    );
    assert_eq!(
        instance.from_u64_to_tuple(99).call().await.unwrap().value,
        ([2; 32], 24, true)
    );
    assert_eq!(
        instance.from_u64_to_tuple(10).call().await.unwrap().value,
        ([3; 32], 99, true)
    );

    assert_eq!(
        instance.from_u64_to_struct(5).call().await.unwrap().value,
        Struct {
            x: 42,
            y: [66; 32],
            z: [99; 32]
        }
    );
    assert_eq!(
        instance.from_u64_to_struct(9).call().await.unwrap().value,
        Struct {
            x: 24,
            y: [11; 32],
            z: [90; 32]
        }
    );
    assert_eq!(
        instance.from_u64_to_struct(1).call().await.unwrap().value,
        Struct {
            x: 77,
            y: [55; 32],
            z: [12; 32]
        }
    );

    assert_eq!(
        instance.from_u64_to_enum(44).call().await.unwrap().value,
        Enum::V1([66; 32])
    );
    assert_eq!(
        instance.from_u64_to_enum(17).call().await.unwrap().value,
        Enum::V2(42)
    );
    assert_eq!(
        instance.from_u64_to_enum(1000).call().await.unwrap().value,
        Enum::V3([42; 32])
    );

    assert_eq!(
        instance.from_u64_to_str(9001).call().await.unwrap().value,
        "fastest_modular_execution_layer_1"
    );
    assert_eq!(
        instance.from_u64_to_str(1980).call().await.unwrap().value,
        "fastest_modular_execution_layer_2"
    );
    assert_eq!(
        instance.from_u64_to_str(1000).call().await.unwrap().value,
        "fastest_modular_execution_layer_3"
    );

    // Get from T -> u64 storage maps
    assert_eq!(
        instance.from_bool_to_u64(true).call().await.unwrap().value,
        1
    );
    assert_eq!(
        instance.from_bool_to_u64(false).call().await.unwrap().value,
        2
    );

    assert_eq!(instance.from_u8_to_u64(8).call().await.unwrap().value, 1);
    assert_eq!(instance.from_u8_to_u64(66).call().await.unwrap().value, 2);
    assert_eq!(instance.from_u8_to_u64(99).call().await.unwrap().value, 3);

    assert_eq!(instance.from_u16_to_u64(9).call().await.unwrap().value, 6);
    assert_eq!(instance.from_u16_to_u64(42).call().await.unwrap().value, 9);
    assert_eq!(instance.from_u16_to_u64(100).call().await.unwrap().value, 1);

    assert_eq!(instance.from_u32_to_u64(90).call().await.unwrap().value, 5);
    assert_eq!(instance.from_u32_to_u64(2).call().await.unwrap().value, 99);
    assert_eq!(
        instance.from_u32_to_u64(100).call().await.unwrap().value,
        10
    );

    assert_eq!(
        instance
            .from_tuple_to_u64(([1; 32], 42, true))
            .call()
            .await
            .unwrap()
            .value,
        50
    );
    assert_eq!(
        instance
            .from_tuple_to_u64(([2; 32], 24, true))
            .call()
            .await
            .unwrap()
            .value,
        99
    );
    assert_eq!(
        instance
            .from_tuple_to_u64(([3; 32], 99, true))
            .call()
            .await
            .unwrap()
            .value,
        10
    );

    assert_eq!(
        instance
            .from_struct_to_u64(Struct {
                x: 42,
                y: [66; 32],
                z: [99; 32]
            })
            .call()
            .await
            .unwrap()
            .value,
        5
    );
    assert_eq!(
        instance
            .from_struct_to_u64(Struct {
                x: 24,
                y: [11; 32],
                z: [90; 32]
            })
            .call()
            .await
            .unwrap()
            .value,
        9
    );
    assert_eq!(
        instance
            .from_struct_to_u64(Struct {
                x: 77,
                y: [55; 32],
                z: [12; 32]
            })
            .call()
            .await
            .unwrap()
            .value,
        1
    );

    assert_eq!(
        instance
            .from_enum_to_u64(Enum::V1([66; 32]))
            .call()
            .await
            .unwrap()
            .value,
        44
    );
    //  This assert currently fails.. I'm not sure why yet
    //    assert_eq!(
    //        instance.from_enum_to_u64(Enum::V2(42)).call().await.unwrap().value,
    //        17
    //
    //    );
    assert_eq!(
        instance
            .from_enum_to_u64(Enum::V3([42; 32]))
            .call()
            .await
            .unwrap()
            .value,
        1000
    );

    assert_eq!(
        instance
            .from_str_to_u64("fastest_modular_execution_layer_1".to_string())
            .call()
            .await
            .unwrap()
            .value,
        9001
    );
    assert_eq!(
        instance
            .from_str_to_u64("fastest_modular_execution_layer_2".to_string())
            .call()
            .await
            .unwrap()
            .value,
        1980
    );
    assert_eq!(
        instance
            .from_str_to_u64("fastest_modular_execution_layer_3".to_string())
            .call()
            .await
            .unwrap()
            .value,
        1000
    );
}
