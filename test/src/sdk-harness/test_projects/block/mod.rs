use fuels::{prelude::*, types::Bits256};
use tai64::Tai64;
use tokio::time::{sleep, Duration};

abigen!(Contract(
    name = "BlockTestContract",
    abi = "out/block-abi.json"
));

async fn get_block_instance() -> (BlockTestContract<Wallet>, ContractId, Provider) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let provider = wallet.provider();
    let id = Contract::load_from(
        "out/block.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = BlockTestContract::new(id.clone(), wallet.clone());

    (instance, id.into(), provider.clone())
}

#[tokio::test]
async fn can_get_block_height() {
    let (instance, _id, _) = get_block_instance().await;
    let block_0 = instance.methods().get_block_height().call().await.unwrap();
    let block_1 = instance.methods().get_block_height().call().await.unwrap();
    let block_2 = instance.methods().get_block_height().call().await.unwrap();

    // Probably consecutive blocks but we may have multiple tx per block so be conservative to
    // guarantee the stability of the test
    assert!(block_1.value <= block_0.value + 1);
    assert!(block_2.value <= block_1.value + 1);
}

#[tokio::test]
async fn can_get_header_hash_of_block() {
    let (instance, _id, _) = get_block_instance().await;
    let block_1 = instance.methods().get_block_height().call().await.unwrap();
    let _block_2 = instance.methods().get_block_height().call().await.unwrap();
    let result = instance
        .methods()
        .get_block_header_hash(block_1.value)
        .call()
        .await
        .unwrap();

    // TODO: when SDK supports getting block-header hash, compare it to hash returned by Sway std::block::block_header_hash()
    assert_ne!(
        result.value,
        Bits256::from_hex_str("0x0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap()
    );
}

#[tokio::test]
async fn can_get_timestamp() {
    let (instance, _id, _) = get_block_instance().await;
    let block_0_time = instance.methods().get_timestamp().call().await.unwrap();
    let now = Tai64::now();

    // This should really be zero in most cases, but be conservative to guarantee the stability of
    // the test
    assert!(now.0 - block_0_time.value <= 1);

    // Wait 1 seconds and request another block
    sleep(Duration::from_secs(1)).await;
    let block_1_time = instance.methods().get_timestamp().call().await.unwrap();

    // The difference should be 1 second in most cases, but be conservative to guarantee the
    // stability of the test
    assert!(
        1 <= block_1_time.value - block_0_time.value
            && block_1_time.value - block_0_time.value <= 2
    );
    // Wait 2 seconds and request another block
    sleep(Duration::from_secs(2)).await;
    let block_2_time = instance.methods().get_timestamp().call().await.unwrap();

    // The difference should be 2 seconds in most cases, but be conservative to guarantee the
    // stability of the test
    assert!(
        2 <= block_2_time.value - block_1_time.value
            && block_2_time.value - block_1_time.value <= 3
    );
}

#[tokio::test]
async fn can_get_timestamp_of_block() {
    let (instance, _id, _) = get_block_instance().await;

    let block_0 = instance
        .methods()
        .get_block_and_timestamp()
        .call()
        .await
        .unwrap();

    sleep(Duration::from_secs(1)).await;
    let block_1 = instance
        .methods()
        .get_block_and_timestamp()
        .call()
        .await
        .unwrap();

    sleep(Duration::from_secs(2)).await;
    let block_2 = instance
        .methods()
        .get_block_and_timestamp()
        .call()
        .await
        .unwrap();

    // Check that the result of `timestamp_of_block` matches the recorded result of `timestamp()`
    // above called via `get_block_and_timestamp`.
    assert_eq!(
        instance
            .methods()
            .get_timestamp_of_block(block_0.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_0.value.1
    );
    assert_eq!(
        instance
            .methods()
            .get_timestamp_of_block(block_1.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_1.value.1
    );
    assert_eq!(
        instance
            .methods()
            .get_timestamp_of_block(block_2.value.0)
            .call()
            .await
            .unwrap()
            .value,
        block_2.value.1
    );
}


#[tokio::test]
async fn can_get_chain_id() {
    let (instance, _id, provider) = get_block_instance().await;

    let id = instance
        .methods()
        .get_chain_id()
        .call()
        .await
        .unwrap();

    assert_eq!(id.value, *provider.consensus_parameters().await.unwrap().chain_id());
}
