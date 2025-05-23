use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageStringContract",
    abi = "test_projects/storage_string/out/release/storage_string-abi.json",
));

async fn setup() -> TestStorageStringContract<Wallet> {
    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(1), None, None),
        Some(node_config),
        None,
    )
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();
    let id = Contract::load_from(
        "test_projects/storage_string/out/release/storage_string.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestStorageStringContract::new(id, wallet)
}

#[tokio::test]
async fn stores_string() {
    let instance = setup().await;

    let input = "Fuel is blazingly fast!";

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        0
    );

    instance
        .methods()
        .store_string(input.into())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        input.as_bytes().len() as u64
    );

    assert_eq!(
        instance.methods().get_string().call().await.unwrap().value,
        Bytes(input.as_bytes().to_vec())
    );
}

#[tokio::test]
async fn stores_long_string() {
    let instance = setup().await;

    // 2060 bytes, max length of URI
    let input = "Nam quis nulla. Integer malesuada. In in enim a arcu imperdiet malesuada. Sed vel lectus. Donec odio urna, tempus molestie, porttitor ut, iaculis quis, sem. Phasellus rhoncus. Aenean id metus id velit ullamcorper pulvinar. Vestibulum fermentum tortor id mi. Pellentesque ipsum. Nulla non arcu lacinia neque faucibus fringilla. Nulla non lectus sed nisl molestie malesuada. Proin in tellus sit amet nibh dignissim sagittis. Vivamus luctus egestas leo. Maecenas sollicitudin. Nullam rhoncus aliquam metus. Etiam egestas wisi a erat.

    Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Nullam feugiat, turpis at pulvinar vulputate, erat libero tristique tellus, nec bibendum odio risus sit amet ante. Aliquam erat volutpat. Nunc auctor. Mauris pretium quam et urna. Fusce nibh. Duis risus. Curabitur sagittis hendrerit ante. Aliquam erat volutpat. Vestibulum erat nulla, ullamcorper nec, rutrum non, nonummy ac, erat. Duis condimentum augue id magna semper rutrum. Nullam justo enim, consectetuer nec, ullamcorper ac, vestibulum in, elit. Proin pede metus, vulputate nec, fermentum fringilla, vehicula vitae, justo. Fusce consectetuer risus a nunc. Aliquam ornare wisi eu metus. Integer pellentesque quam vel velit. Duis pulvinar.

    Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Morbi gravida libero nec velit. Morbi scelerisque luctus velit. Etiam dui sem, fermentum vitae, sagittis id, malesuada in, quam. Proin mattis lacinia justo. Vestibulum facilisis auctor urna. Aliquam in lorem sit amet leo accumsan lacinia. Integer rutrum, orci vestibulum ullamcorper ultricies, lacus quam ultricies odio, vitae placerat pede sem sit amet enim. Phasellus et lorem id felis nonummy placerat. Fusce dui leo, imperdiet in, aliquam sit amet, feugiat eu, orci. Aenean vel massa quis mauris vehicula lacinia. Quisque tincidunt scelerisque libero. Maecenas libero. Etiam dictum tincidunt diam. Donec ipsum massa, ullamcorper in, auctor et, scelerisque sed, est. Suspendisse nisl. Sed convallis magna eu sem. Cras pede libero, dapibus nec, pretium";

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        0
    );

    let tx_policies = TxPolicies::default().with_script_gas_limit(12_000_000);
    instance
        .methods()
        .store_string(input.into())
        .with_tx_policies(tx_policies)
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        input.as_bytes().len() as u64
    );

    assert_eq!(
        instance.methods().get_string().call().await.unwrap().value,
        Bytes(input.as_bytes().to_vec())
    );
}

#[tokio::test]
async fn stores_string_twice() {
    let instance = setup().await;

    let input1 = "Fuel is the fastest modular execution layer";
    let input2 = "Fuel is blazingly fast!";

    instance
        .methods()
        .store_string(input1.into())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        input1.as_bytes().len() as u64
    );

    assert_eq!(
        instance.methods().get_string().call().await.unwrap().value,
        Bytes(input1.as_bytes().to_vec())
    );

    instance
        .methods()
        .store_string(input2.into())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        input2.as_bytes().len() as u64
    );

    assert_eq!(
        instance.methods().get_string().call().await.unwrap().value,
        Bytes(input2.as_bytes().to_vec())
    );
}

#[tokio::test]
async fn clears_bytes() {
    let instance = setup().await;

    let input = "Fuel is blazingly fast!";

    instance
        .methods()
        .store_string(input.into())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        input.as_bytes().len() as u64
    );

    assert!(
        instance
            .methods()
            .clear_string()
            .call()
            .await
            .unwrap()
            .value
    );

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        0
    );
}

#[tokio::test]
async fn get_string_length() {
    let instance = setup().await;

    let input = "Fuel is blazingly fast!";

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        0
    );

    instance
        .methods()
        .store_string(input.into())
        .call()
        .await
        .unwrap();

    assert_eq!(
        instance.methods().stored_len().call().await.unwrap().value,
        input.as_bytes().len() as u64
    );
}
