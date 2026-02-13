use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageVecOfStorageStringContract",
    abi = "out_for_sdk_harness_tests/storage_vec_of_storage_string-abi.json",
));

async fn test_storage_vec_of_storage_string_instance(
) -> TestStorageVecOfStorageStringContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out_for_sdk_harness_tests/storage_vec_of_storage_string.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestStorageVecOfStorageStringContract::new(id.clone(), wallet)
}

// This test proves that https://github.com/FuelLabs/sway/issues/6036 is fixed.
#[tokio::test]
async fn test_push_and_get() {
    let instance = test_storage_vec_of_storage_string_instance().await;

    const NUM_OF_STRINGS: u64 = 10; // Keep it larger then 8, to stress the internal implementation that does % 8.
    let strings = (0..NUM_OF_STRINGS)
        .map(|i| i.to_string())
        .collect::<Vec<_>>();

    for string in &strings {
        let _ = instance.methods().push(string.to_owned()).call().await;
    }

    let returned_count = instance.methods().count().call().await.unwrap().value;

    assert_eq!(returned_count, NUM_OF_STRINGS);

    let mut returned_strings = vec![];
    for i in 0..NUM_OF_STRINGS {
        let returned_string = instance.methods().get(i).call().await.unwrap().value;

        returned_strings.push(returned_string);
    }

    assert_eq!(returned_strings, strings);
}

// TODO: Uncomment this test once https://github.com/FuelLabs/sway/issues/6040 is fixed.
// #[tokio::test]
// async fn test_push_and_insert() {
//     let instance = test_storage_vec_of_storage_string_instance().await;

//     const NUM_OF_STRINGS: u64 = 10; // Keep it larger then 8, to stress the internal implementation that does % 8.
//     let mut strings = (0..NUM_OF_STRINGS).map(|i| i.to_string()).collect::<Vec<_>>();

//     for string in &strings {
//         let _ = instance
//             .methods()
//             .insert(string.to_owned())
//             .call()
//             .await;
//     }

//     let returned_count = instance
//         .methods()
//         .count()
//         .call()
//         .await
//         .unwrap()
//         .value;

//     assert_eq!(returned_count, NUM_OF_STRINGS);

//     let mut returned_strings = vec![];
//     for i in 0..NUM_OF_STRINGS {
//         let returned_string = instance
//             .methods()
//             .get(i)
//             .call()
//             .await
//             .unwrap()
//             .value;

//         returned_strings.push(returned_string);
//     }

//     strings.reverse();

//     assert_eq!(returned_strings, strings);
// }
