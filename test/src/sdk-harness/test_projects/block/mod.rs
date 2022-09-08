use fuels::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

abigen!(
    BlockTestContract,
    "test_projects/block/out/debug/block-abi.json"
);

async fn get_block_instance() -> (BlockTestContract, ContractId) {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/block/out/debug/block.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/block/out/debug/block-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = BlockTestContractBuilder::new(id.to_string(), wallet).build();

    (instance, id.into())
}

#[tokio::test]
async fn can_get_block_height() {
    let (instance, _id) = get_block_instance().await;
    let block_0 = instance.get_block_height().call().await.unwrap();
    let block_1 = instance.get_block_height().call().await.unwrap();
    let block_2 = instance.get_block_height().call().await.unwrap();

    // Consecutive blocks
    assert_eq!(block_1.value, block_0.value + 1);
    assert_eq!(block_2.value, block_1.value + 1);
}

#[tokio::test]
async fn can_get_timestamp() {
    let (instance, _id) = get_block_instance().await;
    let block_0_time = instance.get_timestamp().call().await.unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // This should really be zero in most cases, but be conservative to guarantee stability of the
    // test
    assert!(now.as_secs() - block_0_time.value <= 1);

    // Wait 1 seconds and request another block
    sleep(Duration::from_secs(1)).await;
    let block_1_time = instance.get_timestamp().call().await.unwrap();

    // The difference should be 1 second in most cases, but be conservative to guarantee stability
    // of the test
    assert!(block_1_time.value - block_0_time.value <= 2);

    // Wait 2 seconds and request another block
    sleep(Duration::from_secs(2)).await;
    let block_2_time = instance.get_timestamp().call().await.unwrap();

    // The difference should be 2 seconds in most cases, but be conservative to guarantee stability
    // of the test
    assert!(block_2_time.value - block_1_time.value <= 3);
}

#[tokio::test]
async fn can_get_timestamp_of_block() {
    let (instance, _id) = get_block_instance().await;

    let block_0 = instance.get_block_and_timestamp().call().await.unwrap();

    sleep(Duration::from_secs(1)).await;
    let block_1 = instance.get_block_and_timestamp().call().await.unwrap();

    sleep(Duration::from_secs(2)).await;
    let block_2 = instance.get_block_and_timestamp().call().await.unwrap();

    // Check that the result of `timestamp_of_block` matches the recorded result of `timestamp()`
    // above called via `get_block_and_timestamp`.
    assert_eq!(
        instance
            .get_timestamp_of_block(block_0.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_0.value.1
    );
    assert_eq!(
        instance
            .get_timestamp_of_block(block_1.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_1.value.1
    );
    assert_eq!(
        instance
            .get_timestamp_of_block(block_2.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_2.value.1
    );
}
