use fuel_core::types::fuel_tx::ContractIdExt;
use fuel_vm::consts::VM_MAX_RAM;
use fuels::tx::Bytes32;
use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::*,
    types::{Bits256, ContractId},
};

abigen!(
    Contract(
        name = "TestContextContract",
        abi = "test_projects/context/out/debug/context-abi.json",
    ),
    Contract(
        name = "TestContextCallerContract",
        abi = "test_artifacts/context_caller_contract/out/debug/context_caller_contract-abi.json",
    ),
    Contract(
        name = "FuelCoin",
        abi = "test_projects/token_ops/out/debug/token_ops-abi.json"
    )
);

async fn get_contracts() -> (
    TestContextContract<WalletUnlocked>,
    ContractId,
    TestContextCallerContract<WalletUnlocked>,
    ContractId,
) {
    let wallet = launch_provider_and_get_wallet().await;
    let id_1 = Contract::load_from(
        "test_projects/context/out/debug/context.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();
    let id_2 = Contract::load_from(
        "test_artifacts/context_caller_contract/out/debug/context_caller_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

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
        .tx_params(TxParameters::default().with_gas_limit(1_000_000))
        .call()
        .await
        .unwrap();

    let result = context_instance
        .methods()
        .get_this_balance(Bits256(*caller_id.asset_id(&Bytes32::zeroed())))
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
        .get_balance_of_contract(
            Bits256(*caller_id.asset_id(&Bytes32::zeroed())),
            caller_id.clone(),
        )
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
        .tx_params(TxParameters::default().with_gas_limit(1_000_000))
        .call()
        .await
        .unwrap();

    assert_eq!(
        result.value,
        Bits256(*caller_id.asset_id(&Bytes32::zeroed()))
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
        .tx_params(TxParameters::default().with_gas_limit(1_000_000))
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
        .tx_params(TxParameters::default().with_gas_limit(1_000_000))
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .methods()
        .call_get_global_gas_with_coins(send_amount, context_id)
        .with_contracts(&[&context_instance])
        .tx_params(TxParameters::default().with_gas_limit(1_000_000))
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
