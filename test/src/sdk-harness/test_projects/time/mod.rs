use fuels::prelude::*;
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};

abigen!(Contract(
    name = "TimeTestContract",
    abi = "out/time-abi.json"
));

async fn get_block_instance() -> (TimeTestContract<Wallet>, ContractId, Provider) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let provider = wallet.provider();
    let id = Contract::load_from(
        "out/time.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = TimeTestContract::new(id.clone(), wallet.clone());

    (instance, id.into(), provider.clone())
}

#[tokio::test]
async fn can_get_unix_timestamp() {
    let (instance, _id, _) = get_block_instance().await;
    let block_0_time = instance.methods().get_now().call().await.unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    // This should really be zero in most cases, but be conservative to guarantee the stability of
    // the test
    assert!(now - block_0_time.value.unix <= 1);

    // Wait 1 seconds and request another block
    sleep(Duration::from_secs(1)).await;
    let block_1_time = instance.methods().get_now().call().await.unwrap();

    // The difference should be 1 second in most cases, but be conservative to guarantee the
    // stability of the test
    assert!(
        1 <= block_1_time.value.unix - block_0_time.value.unix
            && block_1_time.value.unix - block_0_time.value.unix <= 2
    );
    // Wait 2 seconds and request another block
    sleep(Duration::from_secs(2)).await;
    let block_2_time = instance.methods().get_now().call().await.unwrap();

    // The difference should be 2 seconds in most cases, but be conservative to guarantee the
    // stability of the test
    assert!(
        2 <= block_2_time.value.unix - block_1_time.value.unix
            && block_2_time.value.unix - block_1_time.value.unix <= 3
    );
}

#[tokio::test]
async fn can_get_unix_timestamp_of_block() {
    let (instance, _id, _) = get_block_instance().await;

    let block_0 = instance
        .methods()
        .get_height_and_time()
        .call()
        .await
        .unwrap();

    sleep(Duration::from_secs(1)).await;
    let block_1 = instance
        .methods()
        .get_height_and_time()
        .call()
        .await
        .unwrap();

    sleep(Duration::from_secs(2)).await;
    let block_2 = instance
        .methods()
        .get_height_and_time()
        .call()
        .await
        .unwrap();

    // Check that the result of `get_height_and_time` matches the recorded result of `Time::now()`
    // above called via `get_height_and_time`.
    assert_eq!(
        instance
            .methods()
            .get_block(block_0.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_0.value.1
    );
    assert_eq!(
        instance
            .methods()
            .get_block(block_1.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_1.value.1
    );
    assert_eq!(
        instance
            .methods()
            .get_block(block_2.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_2.value.1
    );
}

#[tokio::test]
async fn can_convert_to_unix_time() {
    let (instance, _id, _) = get_block_instance().await;

    let (time_1, tia64_1) = instance
        .methods()
        .get_time_and_tia64()
        .call()
        .await
        .unwrap()
        .value;

    sleep(Duration::from_secs(1)).await;
    let (time_2, tia64_2) = instance
        .methods()
        .get_time_and_tia64()
        .call()
        .await
        .unwrap()
        .value;

    sleep(Duration::from_secs(2)).await;
    let (time_3, tia64_3) = instance
        .methods()
        .get_time_and_tia64()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(
        instance
            .methods()
            .from_tia64(tia64_1)
            .call()
            .await
            .unwrap()
            .value,
        time_1
    );

    assert_eq!(
        instance
            .methods()
            .from_tia64(tia64_2)
            .call()
            .await
            .unwrap()
            .value,
        time_2
    );

    assert_eq!(
        instance
            .methods()
            .from_tia64(tia64_3)
            .call()
            .await
            .unwrap()
            .value,
        time_3
    );
}

#[tokio::test]
async fn can_convert_to_tai64_time() {
    let (instance, _id, _) = get_block_instance().await;

    let (time_1, tia64_1) = instance
        .methods()
        .get_time_and_tia64()
        .call()
        .await
        .unwrap()
        .value;

    sleep(Duration::from_secs(1)).await;
    let (time_2, tia64_2) = instance
        .methods()
        .get_time_and_tia64()
        .call()
        .await
        .unwrap()
        .value;

    sleep(Duration::from_secs(2)).await;
    let (time_3, tia64_3) = instance
        .methods()
        .get_time_and_tia64()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(
        instance
            .methods()
            .into_tai64(time_1)
            .call()
            .await
            .unwrap()
            .value,
        tia64_1
    );

    assert_eq!(
        instance
            .methods()
            .into_tai64(time_2)
            .call()
            .await
            .unwrap()
            .value,
        tia64_2
    );

    assert_eq!(
        instance
            .methods()
            .into_tai64(time_3)
            .call()
            .await
            .unwrap()
            .value,
        tia64_3
    );
}
