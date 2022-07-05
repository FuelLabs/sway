use fuels::{prelude::*, tx::ContractId};

abigen!(
    EvmTestContract,
    "test_projects/evm/out/debug/evm-abi.json"
);

async fn get_evm_test_instance() -> (EvmTestContract, ContractId) {
    let wallet = launch_provider_and_get_single_wallet().await;
    let id = Contract::deploy(
        "test_projects/evm/out/debug/evm.bin",
        &wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();
    let instance = EvmTestContract::new(id.to_string(), wallet);

    (instance, id)
}

#[tokio::test]
async fn can_call_from() {
    let (instance, _) = get_evm_test_instance().await;
    
    let raw_address = [6; 32]; // hardcoded in the contract to test calling `from()` with a literal
    let result = instance.evm_address_from_literal().call().await.unwrap();
    assert_eq!(result.value.value[0..12], [0; 12]);
    assert_eq!(result.value.value[12..32], raw_address[12..32]);

    let raw_address = [7; 32];
    let result = instance.evm_address_from_argument(raw_address.into()).call().await.unwrap();
    assert_eq!(result.value.value[0..12], [0; 12]);
    assert_eq!(result.value.value[12..32], raw_address[12..32]);
}
