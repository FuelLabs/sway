use fuels::{prelude::*, tx::ContractId};

abigen!(EvmTestContract, "test_projects/evm/out/debug/evm-abi.json");

async fn get_evm_test_instance() -> (EvmTestContract, ContractId) {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/evm/out/debug/evm.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/evm/out/debug/evm-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = EvmTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn can_call_from_literal() {
    let (instance, _) = get_evm_test_instance().await;

    let raw_address = [6; 32]; // hardcoded in the contract to test calling `from()` with a literal
    let result = instance
        .methods()
        .evm_address_from_literal()
        .call()
        .await
        .unwrap();
    let returned_value = result.value.value;
    assert_eq!(returned_value.0[0..12], [0; 12]);
    assert_eq!(returned_value.0[12..32], raw_address[12..32]);
}

#[tokio::test]
async fn can_call_from_argument() {
    let (instance, _) = get_evm_test_instance().await;

    let raw_address = [7; 32];
    let result = instance
        .methods()
        .evm_address_from_argument(Bits256(raw_address))
        .call()
        .await
        .unwrap();
    let returned_value = result.value.value;
    assert_eq!(returned_value.0[0..12], [0; 12]);
    assert_eq!(returned_value.0[12..32], raw_address[12..32]);
}
