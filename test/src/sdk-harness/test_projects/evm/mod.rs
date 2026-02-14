use fuels::{
    prelude::*,
    types::{Bits256, ContractId, EvmAddress},
};

abigen!(Contract(
    name = "EvmTestContract",
    abi = "out/evm-abi.json"
));

async fn get_evm_test_instance() -> (EvmTestContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out/evm.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = EvmTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn can_call_from_literal() {
    let (instance, _) = get_evm_test_instance().await;

    let result = instance
        .methods()
        .evm_address_from_literal()
        .call()
        .await
        .unwrap();

    assert_eq!(
        EvmAddress::from(
            Bits256::from_hex_str(
                "0x0606060606060606060606060606060606060606060606060606060606060606",
            )
            .unwrap()
        ),
        result.value
    );
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

    assert_eq!(EvmAddress::from(Bits256(raw_address)), result.value);
}
