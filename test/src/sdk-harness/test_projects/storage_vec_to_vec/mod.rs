use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageVecToVecContract",
    abi = "test_projects/storage_vec_to_vec/out/debug/storage_vec_to_vec-abi.json",
));

async fn test_storage_vec_to_vec_instance() -> TestStorageVecToVecContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/storage_vec_to_vec/out/debug/storage_vec_to_vec.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    TestStorageVecToVecContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn test_conversion_u64() {
    let instance = test_storage_vec_to_vec_instance().await;

    let mut test_vec = Vec::<u64>::new();
    test_vec.push(5u64);
    test_vec.push(7u64);
    test_vec.push(9u64);
    test_vec.push(11u64);

    let _ = instance.methods().store_vec_u64(test_vec).call().await;

    let returned_vec = instance
        .methods()
        .read_vec_u64()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(returned_vec.len(), 4);
    assert_eq!(*returned_vec.get(0).unwrap(), 5u64);
    assert_eq!(*returned_vec.get(1).unwrap(), 7u64);
    assert_eq!(*returned_vec.get(2).unwrap(), 9u64);
    assert_eq!(*returned_vec.get(3).unwrap(), 11u64);
}

#[tokio::test]
async fn test_push_u64() {
    let instance = test_storage_vec_to_vec_instance().await;

    let mut test_vec = Vec::<u64>::new();
    test_vec.push(5u64);
    test_vec.push(7u64);
    test_vec.push(9u64);
    test_vec.push(11u64);

    let _ = instance.methods().store_vec_u64(test_vec).call().await;

    let _ = instance.methods().push_vec_u64(13u64).call().await;

    let returned_vec = instance
        .methods()
        .read_vec_u64()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(returned_vec.len(), 5);
    assert_eq!(*returned_vec.get(0).unwrap(), 5u64);
    assert_eq!(*returned_vec.get(1).unwrap(), 7u64);
    assert_eq!(*returned_vec.get(2).unwrap(), 9u64);
    assert_eq!(*returned_vec.get(3).unwrap(), 11u64);
    assert_eq!(*returned_vec.get(4).unwrap(), 13u64);
}

#[tokio::test]
async fn test_pop_u64() {
    let instance = test_storage_vec_to_vec_instance().await;

    let mut test_vec = Vec::<u64>::new();
    test_vec.push(5u64);
    test_vec.push(7u64);
    test_vec.push(9u64);
    test_vec.push(11u64);

    let _ = instance.methods().store_vec_u64(test_vec).call().await;

    assert_eq!(
        11u64,
        instance.methods().pop_vec_u64().call().await.unwrap().value
    );

    let returned_vec = instance
        .methods()
        .read_vec_u64()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(returned_vec.len(), 3);
    assert_eq!(*returned_vec.get(0).unwrap(), 5u64);
    assert_eq!(*returned_vec.get(1).unwrap(), 7u64);
    assert_eq!(*returned_vec.get(2).unwrap(), 9u64);
}

#[tokio::test]
async fn test_conversion_struct() {
    let instance = test_storage_vec_to_vec_instance().await;

    let mut test_vec = Vec::<TestStruct>::new();
    test_vec.push(TestStruct {
        val_1: 0u64,
        val_2: 1u64,
        val_3: 2u64,
    });
    test_vec.push(TestStruct {
        val_1: 1u64,
        val_2: 2u64,
        val_3: 3u64,
    });
    test_vec.push(TestStruct {
        val_1: 2u64,
        val_2: 3u64,
        val_3: 4u64,
    });
    test_vec.push(TestStruct {
        val_1: 3u64,
        val_2: 4u64,
        val_3: 5u64,
    });

    let _ = instance.methods().store_vec_struct(test_vec).call().await;

    let returned_vec = instance
        .methods()
        .read_vec_struct()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(returned_vec.len(), 4);
    assert_eq!(
        *returned_vec.get(0).unwrap(),
        TestStruct {
            val_1: 0u64,
            val_2: 1u64,
            val_3: 2u64
        }
    );
    assert_eq!(
        *returned_vec.get(1).unwrap(),
        TestStruct {
            val_1: 1u64,
            val_2: 2u64,
            val_3: 3u64
        }
    );
    assert_eq!(
        *returned_vec.get(2).unwrap(),
        TestStruct {
            val_1: 2u64,
            val_2: 3u64,
            val_3: 4u64
        }
    );
    assert_eq!(
        *returned_vec.get(3).unwrap(),
        TestStruct {
            val_1: 3u64,
            val_2: 4u64,
            val_3: 5u64
        }
    );
}

#[tokio::test]
async fn test_push_struct() {
    let instance = test_storage_vec_to_vec_instance().await;

    let mut test_vec = Vec::<TestStruct>::new();
    test_vec.push(TestStruct {
        val_1: 0u64,
        val_2: 1u64,
        val_3: 2u64,
    });
    test_vec.push(TestStruct {
        val_1: 1u64,
        val_2: 2u64,
        val_3: 3u64,
    });
    test_vec.push(TestStruct {
        val_1: 2u64,
        val_2: 3u64,
        val_3: 4u64,
    });
    test_vec.push(TestStruct {
        val_1: 3u64,
        val_2: 4u64,
        val_3: 5u64,
    });

    let _ = instance.methods().store_vec_struct(test_vec).call().await;

    let _ = instance
        .methods()
        .push_vec_struct(TestStruct {
            val_1: 4u64,
            val_2: 5u64,
            val_3: 6u64,
        })
        .call()
        .await;

    let returned_vec = instance
        .methods()
        .read_vec_struct()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(returned_vec.len(), 5);
    assert_eq!(
        *returned_vec.get(0).unwrap(),
        TestStruct {
            val_1: 0u64,
            val_2: 1u64,
            val_3: 2u64
        }
    );
    assert_eq!(
        *returned_vec.get(1).unwrap(),
        TestStruct {
            val_1: 1u64,
            val_2: 2u64,
            val_3: 3u64
        }
    );
    assert_eq!(
        *returned_vec.get(2).unwrap(),
        TestStruct {
            val_1: 2u64,
            val_2: 3u64,
            val_3: 4u64
        }
    );
    assert_eq!(
        *returned_vec.get(3).unwrap(),
        TestStruct {
            val_1: 3u64,
            val_2: 4u64,
            val_3: 5u64
        }
    );
    assert_eq!(
        *returned_vec.get(4).unwrap(),
        TestStruct {
            val_1: 4u64,
            val_2: 5u64,
            val_3: 6u64
        }
    );
}

#[tokio::test]
async fn test_pop_struct() {
    let instance = test_storage_vec_to_vec_instance().await;

    let mut test_vec = Vec::<TestStruct>::new();
    test_vec.push(TestStruct {
        val_1: 0u64,
        val_2: 1u64,
        val_3: 2u64,
    });
    test_vec.push(TestStruct {
        val_1: 1u64,
        val_2: 2u64,
        val_3: 3u64,
    });
    test_vec.push(TestStruct {
        val_1: 2u64,
        val_2: 3u64,
        val_3: 4u64,
    });
    test_vec.push(TestStruct {
        val_1: 3u64,
        val_2: 4u64,
        val_3: 5u64,
    });

    let _ = instance.methods().store_vec_struct(test_vec).call().await;

    assert_eq!(
        TestStruct {
            val_1: 3u64,
            val_2: 4u64,
            val_3: 5u64
        },
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

    assert_eq!(returned_vec.len(), 3);
    assert_eq!(
        *returned_vec.get(0).unwrap(),
        TestStruct {
            val_1: 0u64,
            val_2: 1u64,
            val_3: 2u64
        }
    );
    assert_eq!(
        *returned_vec.get(1).unwrap(),
        TestStruct {
            val_1: 1u64,
            val_2: 2u64,
            val_3: 3u64
        }
    );
    assert_eq!(
        *returned_vec.get(2).unwrap(),
        TestStruct {
            val_1: 2u64,
            val_2: 3u64,
            val_3: 4u64
        }
    );
}
