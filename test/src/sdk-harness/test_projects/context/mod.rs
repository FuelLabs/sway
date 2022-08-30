use fuel_vm::consts::VM_MAX_RAM;
use fuels::{prelude::*, tx::ContractId};

abigen!(
    TestContextContract,
    "test_projects/context/out/debug/context-flat-abi.json",
);
abigen!(
    TestContextCallerContract,
    "test_artifacts/context_caller_contract/out/debug/context_caller_contract-flat-abi.json",
);
abigen!(
    FuelCoin,
    "test_projects/token_ops/out/debug/token_ops-flat-abi.json"
);

async fn get_contracts() -> (
    TestContextContract,
    ContractId,
    TestContextCallerContract,
    ContractId,
) {
    let wallet = launch_provider_and_get_wallet().await;
    let id_1 = Contract::deploy(
        "test_projects/context/out/debug/context.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/context/out/debug/context-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let id_2 = Contract::deploy(
        "test_artifacts/context_caller_contract/out/debug/context_caller_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_artifacts/context_caller_contract/out/debug/context_caller_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance_2 =
        TestContextCallerContractBuilder::new(id_2.to_string(), wallet.clone()).build();
    let instance_1 = TestContextContractBuilder::new(id_1.to_string(), wallet.clone()).build();

    (instance_1, id_1.into(), instance_2, id_2.into())
}

#[tokio::test]
async fn can_get_this_balance() {
    let (context_instance, context_id, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    caller_instance
        .call_receive_coins(send_amount, context_id)
        .set_contracts(&[context_id.into()])
        .tx_params(TxParameters::new(None, Some(1_000_000), None, None))
        .call()
        .await
        .unwrap();

    let result = context_instance
        .get_this_balance(caller_id)
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
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = context_instance
        .get_balance_of_contract(caller_id.clone(), caller_id.clone())
        .set_contracts(&[caller_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_msg_value() {
    let (_, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_amount_with_coins(send_amount, context_id)
        .set_contracts(&[context_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_msg_id() {
    let (_, context_id, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_asset_id_with_coins(send_amount, context_id)
        .set_contracts(&[context_id.into()])
        .tx_params(TxParameters::new(None, Some(1_000_000), None, None))
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, caller_id);
}

#[tokio::test]
async fn can_get_msg_gas() {
    let (_, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_gas_with_coins(send_amount, context_id)
        .set_contracts(&[context_id.into()])
        .tx_params(TxParameters::new(Some(0), Some(1_000_000), None, None))
        .call()
        .await
        .unwrap();

    is_within_range(result.value);
}

#[tokio::test]
async fn can_get_global_gas() {
    let (_, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;

    caller_instance
        .mint_coins(send_amount)
        .tx_params(TxParameters::new(None, Some(1_000_000), None, None))
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_global_gas_with_coins(send_amount, context_id)
        .set_contracts(&[context_id.into()])
        .tx_params(TxParameters::new(None, Some(1_000_000), None, None))
        .call()
        .await
        .unwrap();

    is_within_range(result.value);
}

fn is_within_range(n: u64) -> bool {
    if n <= 0 || n > VM_MAX_RAM {
        false
    } else {
        true
    }
}
