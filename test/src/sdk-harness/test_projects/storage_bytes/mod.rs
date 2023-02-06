use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageBytesContract",
    abi = "test_projects/storage_bytes/out/debug/storage_bytes-abi.json",
));

async fn test_storage_bytes_instance() -> TestStorageBytesContract {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/storage_bytes/out/debug/storage_bytes.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/storage_bytes/out/debug/storage_bytes-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    TestStorageBytesContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn returns_empty_bytes() {
    let instance = test_storage_bytes_instance().await;

    let input = vec![];

    assert_eq!(instance.methods().len().call().await.unwrap().value, 0);

    let _ = instance
        .methods()
        .assert_stored_bytes(input.clone())
        .call()
        .await;
}

#[tokio::test]
async fn stores_byte() {
    let instance = test_storage_bytes_instance().await;

    let input = vec![1u8];

    assert_eq!(instance.methods().len().call().await.unwrap().value, 0);

    let _ = instance.methods().store_bytes(input.clone()).call().await;

    assert_eq!(instance.methods().len().call().await.unwrap().value, 1);

    let _ = instance
        .methods()
        .assert_stored_bytes(input.clone())
        .call()
        .await;
}

#[tokio::test]
async fn stores_8_bytes() {
    let instance = test_storage_bytes_instance().await;

    let input = vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8];

    assert_eq!(instance.methods().len().call().await.unwrap().value, 0);

    let _ = instance.methods().store_bytes(input.clone()).call().await;

    assert_eq!(instance.methods().len().call().await.unwrap().value, 8);

    let _ = instance.methods().assert_stored_bytes(input).call().await;
}

#[tokio::test]
async fn stores_32_bytes() {
    let instance = test_storage_bytes_instance().await;

    let input = vec![
        1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 1u8, 2u8,
        3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8,
    ];

    assert_eq!(instance.methods().len().call().await.unwrap().value, 0);

    let _ = instance.methods().store_bytes(input.clone()).call().await;

    assert_eq!(instance.methods().len().call().await.unwrap().value, 32);

    let _ = instance.methods().assert_stored_bytes(input).call().await;
}

#[tokio::test]
async fn stores_string_as_bytes() {
    let instance = test_storage_bytes_instance().await;

    let input = String::from("Fuel is blazingly fast!");

    assert_eq!(instance.methods().len().call().await.unwrap().value, 0);

    let _ = instance
        .methods()
        .store_bytes(input.clone().as_bytes().into())
        .call()
        .await;

    assert_eq!(
        instance.methods().len().call().await.unwrap().value,
        input.clone().as_bytes().len() as u64
    );

    let _ = instance
        .methods()
        .assert_stored_bytes(input.as_bytes().into())
        .call()
        .await;
}

#[tokio::test]
async fn stores_long_string_as_bytes() {
    let instance = test_storage_bytes_instance().await;

    // 2060 bytes
    let input = String::from("Nam quis nulla. Integer malesuada. In in enim a arcu imperdiet malesuada. Sed vel lectus. Donec odio urna, tempus molestie, porttitor ut, iaculis quis, sem. Phasellus rhoncus. Aenean id metus id velit ullamcorper pulvinar. Vestibulum fermentum tortor id mi. Pellentesque ipsum. Nulla non arcu lacinia neque faucibus fringilla. Nulla non lectus sed nisl molestie malesuada. Proin in tellus sit amet nibh dignissim sagittis. Vivamus luctus egestas leo. Maecenas sollicitudin. Nullam rhoncus aliquam metus. Etiam egestas wisi a erat.

    Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Nullam feugiat, turpis at pulvinar vulputate, erat libero tristique tellus, nec bibendum odio risus sit amet ante. Aliquam erat volutpat. Nunc auctor. Mauris pretium quam et urna. Fusce nibh. Duis risus. Curabitur sagittis hendrerit ante. Aliquam erat volutpat. Vestibulum erat nulla, ullamcorper nec, rutrum non, nonummy ac, erat. Duis condimentum augue id magna semper rutrum. Nullam justo enim, consectetuer nec, ullamcorper ac, vestibulum in, elit. Proin pede metus, vulputate nec, fermentum fringilla, vehicula vitae, justo. Fusce consectetuer risus a nunc. Aliquam ornare wisi eu metus. Integer pellentesque quam vel velit. Duis pulvinar.
    
    Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Morbi gravida libero nec velit. Morbi scelerisque luctus velit. Etiam dui sem, fermentum vitae, sagittis id, malesuada in, quam. Proin mattis lacinia justo. Vestibulum facilisis auctor urna. Aliquam in lorem sit amet leo accumsan lacinia. Integer rutrum, orci vestibulum ullamcorper ultricies, lacus quam ultricies odio, vitae placerat pede sem sit amet enim. Phasellus et lorem id felis nonummy placerat. Fusce dui leo, imperdiet in, aliquam sit amet, feugiat eu, orci. Aenean vel massa quis mauris vehicula lacinia. Quisque tincidunt scelerisque libero. Maecenas libero. Etiam dictum tincidunt diam. Donec ipsum massa, ullamcorper in, auctor et, scelerisque sed, est. Suspendisse nisl. Sed convallis magna eu sem. Cras pede libero, dapibus nec, pretium");

    assert_eq!(instance.methods().len().call().await.unwrap().value, 0);

    let tx_params = TxParameters::new(None, Some(12_000_000), None);
    let _ = instance
        .methods()
        .store_bytes(input.clone().as_bytes().into())
        .tx_params(tx_params)
        .call()
        .await;

    assert_eq!(
        instance.methods().len().call().await.unwrap().value,
        input.clone().as_bytes().len() as u64
    );

    let _ = instance
        .methods()
        .assert_stored_bytes(input.as_bytes().into())
        .call()
        .await;
}

#[tokio::test]
async fn stores_string_twice() {
    let instance = test_storage_bytes_instance().await;

    let input1 = String::from("Fuel is the fastest modular execution layer");
    let input2 = String::from("Fuel is blazingly fast!");

    let _ = instance
        .methods()
        .store_bytes(input1.clone().as_bytes().into())
        .call()
        .await;

    assert_eq!(
        instance.methods().len().call().await.unwrap().value,
        input1.clone().as_bytes().len() as u64
    );

    let _ = instance
        .methods()
        .assert_stored_bytes(input1.as_bytes().into())
        .call()
        .await;

    let _ = instance
        .methods()
        .store_bytes(input2.clone().as_bytes().into())
        .call()
        .await;

    assert_eq!(
        instance.methods().len().call().await.unwrap().value,
        input2.clone().as_bytes().len() as u64
    );

    let _ = instance
        .methods()
        .assert_stored_bytes(input2.as_bytes().into())
        .call()
        .await;
}
