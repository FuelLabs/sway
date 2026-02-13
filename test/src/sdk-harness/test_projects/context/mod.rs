use fuel_vm::consts::VM_MAX_RAM;
use fuels::{
    prelude::*,
    tx::ContractIdExt,
    types::{Bits256, SubAssetId, ContractId},
};

abigen!(
    Contract(
        name = "TestContextContract",
        abi = "out_for_sdk_harness_tests/context-abi.json",
    ),
    Contract(
        name = "TestContextCallerContract",
        abi = "out_for_sdk_harness_tests/context_caller_contract-abi.json",
    ),
    Contract(
        name = "FuelCoin",
        abi = "out_for_sdk_harness_tests/asset_ops-abi.json"
    )
);

async fn get_contracts() -> (
    TestContextContract<Wallet>,
    ContractId,
    TestContextCallerContract<Wallet>,
    ContractId,
) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id_1 = Contract::load_from(
        "out_for_sdk_harness_tests/context.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let id_2 = Contract::load_from(
        "out_for_sdk_harness_tests/context_caller_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let instance_2 = TestContextCallerContract::new(id_2.clone(), wallet.clone());
    let instance_1 = TestContextContract::new(id_1.clone(), wallet.clone());

    (instance_1, id_1.into(), instance_2, id_2.into())
}

#[tokio::test]
async fn can_get_this_balance() {
    let (context_instance, context_id, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;

    caller_instance
        .methods()
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    caller_instance
        .methods()
        .call_receive_coins(send_amount, context_id)
        .with_contracts(&[&context_instance])
        .call()
        .await
        .unwrap();

    let result = context_instance
        .methods()
        .get_this_balance(Bits256(*caller_id.asset_id(&SubAssetId::zeroed())))
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_balance_of_contract() {
    let (context_instance, _, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;

    caller_instance
        .methods()
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = context_instance
        .methods()
        .get_balance_of_contract(Bits256(*caller_id.asset_id(&SubAssetId::zeroed())), caller_id)
        .with_contracts(&[&caller_instance])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_msg_value() {
    let (context_instance, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;

    caller_instance
        .methods()
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .methods()
        .call_get_amount_with_coins(send_amount, context_id)
        .with_contracts(&[&context_instance])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_msg_id() {
    let (context_instance, context_id, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;

    caller_instance
        .methods()
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .methods()
        .call_get_asset_id_with_coins(send_amount, context_id)
        .with_contracts(&[&context_instance])
        .call()
        .await
        .unwrap();

    assert_eq!(
        result.value,
        Bits256(*caller_id.asset_id(&SubAssetId::zeroed()))
    );
}

#[tokio::test]
async fn can_get_msg_gas() {
    let (context_instance, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;

    caller_instance
        .methods()
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .methods()
        .call_get_gas_with_coins(send_amount, context_id)
        .with_contracts(&[&context_instance])
        .call()
        .await
        .unwrap();

    is_within_range(result.value);
}

#[tokio::test]
async fn can_get_global_gas() {
    let (context_instance, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;

    caller_instance
        .methods()
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .methods()
        .call_get_global_gas_with_coins(send_amount, context_id)
        .with_contracts(&[&context_instance])
        .call()
        .await
        .unwrap();

    is_within_range(result.value);
}

fn is_within_range(n: u64) -> bool {
    n > 0 && n <= VM_MAX_RAM
}
