use fuels::{prelude::*, types::Bits256};

abigen!(Contract(
    name = "TestStorageAccessContract",
    abi = "test_projects/storage_access/out/release/storage_access-abi.json",
));

async fn test_storage_access_instance() -> TestStorageAccessContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/storage_access/out/release/storage_access.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestStorageAccessContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn simple_access() {
    let methods = test_storage_access_instance().await.methods();

    let input = 42;
    assert_eq!(
        methods
            .write_and_read_u64(input)
            .call()
            .await
            .unwrap()
            .value,
        input
    );

    let input = Bits256([1; 32]);
    assert_eq!(
        methods
            .write_and_read_b256(input)
            .call()
            .await
            .unwrap()
            .value,
        input
    );
}

#[tokio::test]
async fn struct_access_simple() {
    let methods = test_storage_access_instance().await.methods();
    let input = Simple {
        x: 0,
        y: 0,
        b: Bits256([1; 32]),
        z: 69,
        w: 0,
    };
    assert_eq!(
        methods
            .write_and_read_struct_simple(input.clone())
            .call()
            .await
            .unwrap()
            .value,
        input
    );
}

#[tokio::test]
async fn struct_access() {
    let methods = test_storage_access_instance().await.methods();

    let input = S {
        a: 1,
        b: Bits256([2; 32]),
        c: T {
            x: 3,
            y: Bits256([4; 32]),
            z: M {
                u: Bits256([5; 32]),
                v: 8,
            },
        },
        d: Bits256([6; 32]),
    };

    assert_eq!(
        methods
            .write_and_read_struct_1(input.clone())
            .call()
            .await
            .unwrap()
            .value,
        input
    );

    assert_eq!(
        methods
            .write_and_read_struct_2(input.clone())
            .call()
            .await
            .unwrap()
            .value,
        input
    );
}

#[tokio::test]
async fn map_access() {
    let methods = test_storage_access_instance().await.methods();

    let (key1, key2, key3) = (42, 69, 99);
    let (value1, value2, value3) = (1, 2, 3);
    let _ = methods.map_write(key1, value1).call().await;
    let _ = methods.map_write(key2, value2).call().await;
    let _ = methods.map_write(key3, value3).call().await;

    assert_eq!(
        methods.map_read(key1).call().await.unwrap().value,
        Some(value1)
    );
    assert_eq!(
        methods.map_read(key2).call().await.unwrap().value,
        Some(value2)
    );
    assert_eq!(
        methods.map_read(key3).call().await.unwrap().value,
        Some(value3)
    );
    assert_eq!(methods.map_read(0).call().await.unwrap().value, None);
}

#[tokio::test]
async fn maps_in_struct_access() {
    let methods = test_storage_access_instance().await.methods();

    let (key1, key2, key3) = ((42, 24), (69, 96), (99, 88));
    let (value1, value2, value3) = ((1, 4), (2, 5), (3, 6));
    let _ = methods.map_in_struct_write(key1, value1).call().await;
    let _ = methods.map_in_struct_write(key2, value2).call().await;
    let _ = methods.map_in_struct_write(key3, value3).call().await;

    assert_eq!(
        methods.map_in_struct_read(key1).call().await.unwrap().value,
        (Some(value1.0), Some(value1.1))
    );
    assert_eq!(
        methods.map_in_struct_read(key2).call().await.unwrap().value,
        (Some(value2.0), Some(value2.1))
    );
    assert_eq!(
        methods.map_in_struct_read(key3).call().await.unwrap().value,
        (Some(value3.0), Some(value3.1))
    );
    assert_eq!(
        methods
            .map_in_struct_read((0, 0))
            .call()
            .await
            .unwrap()
            .value,
        (None, None)
    );
}

#[tokio::test]
async fn clears_storage_key() {
    let methods = test_storage_access_instance().await.methods();

    assert!(methods.clears_storage_key().call().await.unwrap().value);
}
