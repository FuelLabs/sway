use fuel_tx::{ContractId, Salt};
use fuel_vm::consts::VM_MAX_RAM;
use fuels_abigen_macro::abigen;
use fuels_contract::contract::Contract;
use fuels_contract::parameters::TxParameters;
use fuels_signers::util::test_helpers::setup_test_provider_and_wallet;

abigen!(
    TestContextContract,
    "test_projects/context/out/debug/context-abi.json",
);
abigen!(
    TestContextCallerContract,
    "test_artifacts/context_caller_contract/out/debug/context_caller_contract-abi.json",
);
abigen!(
    FuelCoin,
    "test_projects/token_ops/out/debug/token_ops-abi.json"
);

async fn get_contracts() -> (
    TestContextContract,
    ContractId,
    TestContextCallerContract,
    ContractId,
) {
    let salt = Salt::from([0u8; 32]);
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let compiled_1 =
        Contract::load_sway_contract("test_projects/context/out/debug/context.bin", salt).unwrap();
    let compiled_2 = Contract::load_sway_contract(
        "test_artifacts/context_caller_contract/out/debug/context_caller_contract.bin",
        salt,
    )
    .unwrap();

    let id_1 = Contract::deploy(&compiled_1, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    let id_2 = Contract::deploy(&compiled_2, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance_2 =
        TestContextCallerContract::new(id_2.to_string(), provider.clone(), wallet.clone());
    let instance_1 = TestContextContract::new(id_1.to_string(), provider.clone(), wallet.clone());

    (instance_1, id_1, instance_2, id_2)
}

#[tokio::test]
async fn can_get_this_balance() {
    let (context_instance, context_id, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;

    let context_sway_id = testcontextcallercontract_mod::ContractId {
        value: context_id.into(),
    };
    let caller_sway_id = testcontextcontract_mod::ContractId {
        value: caller_id.into(),
    };

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    caller_instance
        .call_receive_coins(send_amount, context_sway_id)
        .set_contracts(&[context_id])
        .tx_params(TxParameters::new(None, Some(1_000_000), None))
        .call()
        .await
        .unwrap();

    let result = context_instance
        .get_this_balance(caller_sway_id)
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_balance_of_contract() {
    let (context_instance, _, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;
    let target = testcontextcontract_mod::ContractId {
        value: caller_id.into(),
    };

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = context_instance
        .get_balance_of_contract(target.clone(), target.clone())
        .set_contracts(&[caller_id])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_msg_value() {
    let (_, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;
    let context_sway_id = testcontextcallercontract_mod::ContractId {
        value: context_id.into(),
    };

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_amount_with_coins(send_amount, context_sway_id)
        .set_contracts(&[context_id])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, send_amount);
}

#[tokio::test]
async fn can_get_msg_id() {
    let (_, context_id, caller_instance, caller_id) = get_contracts().await;
    let send_amount = 42;
    let caller_sway_id = testcontextcallercontract_mod::ContractId {
        value: caller_id.into(),
    };
    let context_sway_id = testcontextcallercontract_mod::ContractId {
        value: context_id.into(),
    };

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_asset_id_with_coins(send_amount, context_sway_id)
        .set_contracts(&[caller_id, context_id])
        .tx_params(TxParameters::new(None, Some(1_000_000), None))
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, caller_sway_id);
}

#[tokio::test]
async fn can_get_msg_gas() {
    let (_, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;
    let context_sway_id = testcontextcallercontract_mod::ContractId {
        value: context_id.into(),
    };

    caller_instance
        .mint_coins(send_amount)
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_gas_with_coins(send_amount, context_sway_id)
        .set_contracts(&[context_id])
        .tx_params(TxParameters::new(Some(0), Some(1_000_000), None))
        .call()
        .await
        .unwrap();

    is_within_range(result.value);
}

#[tokio::test]
async fn can_get_global_gas() {
    let (_, context_id, caller_instance, _) = get_contracts().await;
    let send_amount = 11;
    let context_sway_id = testcontextcallercontract_mod::ContractId {
        value: context_id.into(),
    };

    caller_instance
        .mint_coins(send_amount)
        .tx_params(TxParameters::new(None, Some(1_000_000), None))
        .call()
        .await
        .unwrap();

    let result = caller_instance
        .call_get_global_gas_with_coins(send_amount, context_sway_id)
        .set_contracts(&[context_id])
        .tx_params(TxParameters::new(None, Some(1_000_000), None))
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
