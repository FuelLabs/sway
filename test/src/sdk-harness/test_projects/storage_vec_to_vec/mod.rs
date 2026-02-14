use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageVecToVecContract",
    abi = "out/storage_vec_to_vec-abi.json",
));

async fn test_storage_vec_to_vec_instance() -> TestStorageVecToVecContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out/storage_vec_to_vec.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestStorageVecToVecContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn test_conversion_u64() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![5, 7, 9, 11];

    let _ = instance
        .methods()
        .store_vec_u64(test_vec.clone())
        .call()
        .await;

    let returned_vec = instance
        .methods()
        .read_vec_u64()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(returned_vec.len(), 4);
    assert_eq!(returned_vec, test_vec);
}

#[tokio::test]
async fn test_push_u64() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![5, 7, 9, 11];

    let _ = instance
        .methods()
        .store_vec_u64(test_vec.clone())
        .call()
        .await;

    let _ = instance.methods().push_vec_u64(13).call().await;

    let returned_vec = instance
        .methods()
        .read_vec_u64()
        .call()
        .await
        .unwrap()
        .value;

    let mut expected_vec = test_vec;
    expected_vec.push(13);

    assert_eq!(returned_vec.len(), 5);
    assert_eq!(returned_vec, expected_vec);
}

#[tokio::test]
async fn test_pop_u64() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![5, 7, 9, 11];

    let _ = instance
        .methods()
        .store_vec_u64(test_vec.clone())
        .call()
        .await;

    assert_eq!(
        11,
        instance.methods().pop_vec_u64().call().await.unwrap().value
    );

    let returned_vec = instance
        .methods()
        .read_vec_u64()
        .call()
        .await
        .unwrap()
        .value;

    let mut expected_vec = test_vec;
    expected_vec.pop();

    assert_eq!(returned_vec.len(), 3);
    assert_eq!(returned_vec, expected_vec);
}

#[tokio::test]
async fn test_conversion_struct() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![
        TestStruct {
            val_1: 0,
            val_2: 1,
            val_3: 2,
        },
        TestStruct {
            val_1: 1,
            val_2: 2,
            val_3: 3,
        },
        TestStruct {
            val_1: 2,
            val_2: 3,
            val_3: 4,
        },
        TestStruct {
            val_1: 3,
            val_2: 4,
            val_3: 5,
        },
    ];

    let _ = instance
        .methods()
        .store_vec_struct(test_vec.clone())
        .call()
        .await;

    let returned_vec = instance
        .methods()
        .read_vec_struct()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(returned_vec.len(), 4);
    assert_eq!(returned_vec, test_vec);
}

#[tokio::test]
async fn test_push_struct() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![
        TestStruct {
            val_1: 0,
            val_2: 1,
            val_3: 2,
        },
        TestStruct {
            val_1: 1,
            val_2: 2,
            val_3: 3,
        },
        TestStruct {
            val_1: 2,
            val_2: 3,
            val_3: 4,
        },
        TestStruct {
            val_1: 3,
            val_2: 4,
            val_3: 5,
        },
    ];

    let test_struct = TestStruct {
        val_1: 4,
        val_2: 5,
        val_3: 6,
    };

    let _ = instance
        .methods()
        .store_vec_struct(test_vec.clone())
        .call()
        .await;

    let _ = instance
        .methods()
        .push_vec_struct(test_struct.clone())
        .call()
        .await;

    let returned_vec = instance
        .methods()
        .read_vec_struct()
        .call()
        .await
        .unwrap()
        .value;

    let mut expected_vec = test_vec;
    expected_vec.push(test_struct);

    assert_eq!(returned_vec.len(), 5);
    assert_eq!(returned_vec, expected_vec);
}

#[tokio::test]
async fn test_pop_struct() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_struct = TestStruct {
        val_1: 3,
        val_2: 4,
        val_3: 5,
    };
    let test_vec = vec![
        TestStruct {
            val_1: 0,
            val_2: 1,
            val_3: 2,
        },
        TestStruct {
            val_1: 1,
            val_2: 2,
            val_3: 3,
        },
        TestStruct {
            val_1: 2,
            val_2: 3,
            val_3: 4,
        },
        test_struct.clone(),
    ];

    let _ = instance
        .methods()
        .store_vec_struct(test_vec.clone())
        .call()
        .await;

    assert_eq!(
        test_struct,
        instance
            .methods()
            .pop_vec_struct()
            .call()
            .await
            .unwrap()
            .value
    );

    let returned_vec = instance
        .methods()
        .read_vec_struct()
        .call()
        .await
        .unwrap()
        .value;

    let mut expected_vec = test_vec;
    expected_vec.pop();

    assert_eq!(returned_vec.len(), 3);
    assert_eq!(returned_vec, expected_vec);
}

#[tokio::test]
async fn test_conversion_u8() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![5u8, 7u8, 9u8, 11u8];

    let _ = instance
        .methods()
        .store_vec_u8(test_vec.clone())
        .call()
        .await;

    let returned_vec = instance.methods().read_vec_u8().call().await.unwrap().value;

    assert_eq!(returned_vec.len(), 4);
    assert_eq!(returned_vec, test_vec);
}

#[tokio::test]
async fn test_push_u8() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![5u8, 7u8, 9u8, 11u8];

    let _ = instance
        .methods()
        .store_vec_u8(test_vec.clone())
        .call()
        .await;

    let _ = instance.methods().push_vec_u8(13u8).call().await;

    let returned_vec = instance.methods().read_vec_u8().call().await.unwrap().value;

    let mut expected_vec = test_vec;
    expected_vec.push(13u8);

    assert_eq!(returned_vec.len(), 5);
    assert_eq!(returned_vec, expected_vec);
}

#[tokio::test]
async fn test_pop_u8() {
    let instance = test_storage_vec_to_vec_instance().await;

    let test_vec = vec![5u8, 7u8, 9u8, 11u8];

    let _ = instance
        .methods()
        .store_vec_u8(test_vec.clone())
        .call()
        .await;

    assert_eq!(
        11u8,
        instance.methods().pop_vec_u8().call().await.unwrap().value
    );

    let returned_vec = instance.methods().read_vec_u8().call().await.unwrap().value;

    let mut expected_vec = test_vec;
    expected_vec.pop();

    assert_eq!(returned_vec.len(), 3);
    assert_eq!(returned_vec, expected_vec);
}
