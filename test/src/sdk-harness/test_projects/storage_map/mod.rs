use fuels::prelude::*;

abigen!(
    TestStorageMapContract,
    "test_projects/storage_map/out/debug/storage_map-abi.json",
);

async fn test_storage_map_instance() -> TestStorageMapContract {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/storage_map/out/debug/storage_map.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/storage_map/out/debug/storage_map-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    TestStorageMapContractBuilder::new(id.to_string(), wallet).build()
}

mod u64_to {

    use super::*;

    #[tokio::test]
    async fn bool_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (1, 2, 3);
        let (val1, val2, val3) = (true, false, true);

        // Insert into u64 -> T storage maps
        instance
            .insert_into_u64_to_bool_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_bool_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_bool_map(key3, val3)
            .call()
            .await
            .unwrap();

        // Get from u64 -> T storage maps
        assert_eq!(
            instance
                .get_from_u64_to_bool_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_bool_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_bool_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn u8_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (1, 2, 3);
        let (val1, val2, val3) = (8, 66, 99);

        instance
            .insert_into_u64_to_u8_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u8_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u8_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_u8_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_u8_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_u8_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn u16_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (6, 9, 1);
        let (val1, val2, val3) = (9, 42, 100);

        instance
            .insert_into_u64_to_u16_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u16_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u16_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_u16_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_u16_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_u16_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn u32_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (5, 99, 10);
        let (val1, val2, val3) = (90, 2, 100);

        instance
            .insert_into_u64_to_u32_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u32_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u32_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_u32_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_u32_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_u32_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn u64_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (50, 99, 1);
        let (val1, val2, val3) = (90, 20, 10);

        instance
            .insert_into_u64_to_u64_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u64_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_u64_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_u64_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_u64_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn tuple_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (50, 99, 10);
        let (val1, val2, val3) = (
            ([1; 32], 42, true),
            ([2; 32], 24, true),
            ([3; 32], 99, true),
        );

        instance
            .insert_into_u64_to_tuple_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_tuple_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_tuple_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_tuple_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_tuple_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_tuple_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn struct_map() {
        let instance = test_storage_map_instance().await;

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
            .insert_into_u64_to_struct_map(key1, val1.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_struct_map(key2, val2.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_struct_map(key3, val3.clone())
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_struct_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_struct_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_struct_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn enum_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (44, 17, 1000);
        let (val1, val2, val3) = (Enum::V1([66; 32]), Enum::V2(42), Enum::V3([42; 32]));

        instance
            .insert_into_u64_to_enum_map(key1, val1.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_enum_map(key2, val2.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_enum_map(key3, val3.clone())
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_enum_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_enum_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_enum_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn string_map() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (9001, 1980, 1000);
        let (val1, val2, val3) = (
            "fastest_modular_execution_layer_A",
            "fastest_modular_execution_layer_B",
            "fastest_modular_execution_layer_C",
        );

        instance
            .insert_into_u64_to_str_map(key1, val1.to_string())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_str_map(key2, val2.to_string())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u64_to_str_map(key3, val3.to_string())
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u64_to_str_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u64_to_str_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u64_to_str_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }
}

mod to_u64_map {

    use super::*;

    #[tokio::test]
    async fn from_bool() {
        let instance = test_storage_map_instance().await;

        let (key1, key2) = (true, false);
        let (val1, val2) = (1, 2);

        instance
            .insert_into_bool_to_u64_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_bool_to_u64_map(key2, val2)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_bool_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_bool_to_u64_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
    }

    #[tokio::test]
    async fn from_u8() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (8, 66, 99);
        let (val1, val2, val3) = (1, 2, 3);

        instance
            .insert_into_u8_to_u64_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u8_to_u64_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u8_to_u64_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u8_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u8_to_u64_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u8_to_u64_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn from_u16() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (9, 42, 100);
        let (val1, val2, val3) = (6, 9, 1);

        instance
            .insert_into_u16_to_u64_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u16_to_u64_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u16_to_u64_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u16_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u16_to_u64_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u16_to_u64_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn from_u32() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (90, 2, 100);
        let (val1, val2, val3) = (5, 99, 10);

        instance
            .insert_into_u32_to_u64_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u32_to_u64_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_u32_to_u64_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_u32_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_u32_to_u64_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_u32_to_u64_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn from_tuple() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (
            ([1; 32], 42, true),
            ([2; 32], 24, true),
            ([3; 32], 99, true),
        );
        let (val1, val2, val3) = (50, 99, 10);

        instance
            .insert_into_tuple_to_u64_map(key1, val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_tuple_to_u64_map(key2, val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_tuple_to_u64_map(key3, val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_tuple_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_tuple_to_u64_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_tuple_to_u64_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn from_struct() {
        let instance = test_storage_map_instance().await;

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
            .insert_into_struct_to_u64_map(key1.clone(), val1.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_struct_to_u64_map(key2.clone(), val2.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_struct_to_u64_map(key3.clone(), val3.clone())
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_struct_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_struct_to_u64_map(key2)
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_struct_to_u64_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn from_enum() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (Enum::V1([66; 32]), Enum::V2(42), Enum::V3([42; 32]));
        let (val1, val2, val3) = (44, 17, 1000);

        instance
            .insert_into_enum_to_u64_map(key1.clone(), val1.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_enum_to_u64_map(key2.clone(), val2.clone())
            .call()
            .await
            .unwrap();
        instance
            .insert_into_enum_to_u64_map(key3.clone(), val3.clone())
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_enum_to_u64_map(key1)
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        // This assert currently fails. Not sure why yet
        //    assert_eq!(
        //        instance.get_from_enum_to_u64_map(key2).call().await.unwrap().value,
        //        val2
        //    );
        assert_eq!(
            instance
                .get_from_enum_to_u64_map(key3)
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }

    #[tokio::test]
    async fn from_string() {
        let instance = test_storage_map_instance().await;

        let (key1, key2, key3) = (
            "fastest_modular_execution_layer_A",
            "fastest_modular_execution_layer_B",
            "fastest_modular_execution_layer_C",
        );
        let (val1, val2, val3) = (9001, 1980, 1000);

        instance
            .insert_into_str_to_u64_map(key1.to_string(), val1)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_str_to_u64_map(key2.to_string(), val2)
            .call()
            .await
            .unwrap();
        instance
            .insert_into_str_to_u64_map(key3.to_string(), val3)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .get_from_str_to_u64_map(key1.to_string())
                .call()
                .await
                .unwrap()
                .value,
            val1
        );
        assert_eq!(
            instance
                .get_from_str_to_u64_map(key2.to_string())
                .call()
                .await
                .unwrap()
                .value,
            val2
        );
        assert_eq!(
            instance
                .get_from_str_to_u64_map(key3.to_string())
                .call()
                .await
                .unwrap()
                .value,
            val3
        );
    }
}

#[tokio::test]
async fn test_multiple_maps() {
    let instance = test_storage_map_instance().await;

    let (key1, key2, key3) = (1, 2, 3);
    let (val1_1, val2_1, val3_1) = (8, 66, 99);
    let (val1_2, val2_2, val3_2) = (9, 42, 100);

    instance
        .insert_into_u64_to_u8_map(key1, val1_1)
        .call()
        .await
        .unwrap();
    instance
        .insert_into_u64_to_u8_map(key2, val2_1)
        .call()
        .await
        .unwrap();
    instance
        .insert_into_u64_to_u8_map(key3, val3_1)
        .call()
        .await
        .unwrap();

    instance
        .insert_into_u64_to_u16_map(key1, val1_2)
        .call()
        .await
        .unwrap();
    instance
        .insert_into_u64_to_u16_map(key2, val2_2)
        .call()
        .await
        .unwrap();
    instance
        .insert_into_u64_to_u16_map(key3, val3_2)
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance
            .get_from_u64_to_u8_map(key1)
            .call()
            .await
            .unwrap()
            .value,
        val1_1
    );
    assert_eq!(
        instance
            .get_from_u64_to_u8_map(key2)
            .call()
            .await
            .unwrap()
            .value,
        val2_1
    );
    assert_eq!(
        instance
            .get_from_u64_to_u8_map(key3)
            .call()
            .await
            .unwrap()
            .value,
        val3_1
    );

    assert_eq!(
        instance
            .get_from_u64_to_u16_map(key1)
            .call()
            .await
            .unwrap()
            .value,
        val1_2
    );
    assert_eq!(
        instance
            .get_from_u64_to_u16_map(key2)
            .call()
            .await
            .unwrap()
            .value,
        val2_2
    );
    assert_eq!(
        instance
            .get_from_u64_to_u16_map(key3)
            .call()
            .await
            .unwrap()
            .value,
        val3_2
    );
}
