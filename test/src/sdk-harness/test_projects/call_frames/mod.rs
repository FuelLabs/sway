use fuel_vm::consts::VM_MAX_RAM;
use fuels::{prelude::*, tx::ContractId};

use sha2::{Digest, Sha256};

abigen!(
    CallFramesTestContract,
    "test_projects/call_frames/out/debug/call_frames-abi.json"
);

async fn get_call_frames_instance() -> (CallFramesTestContract, ContractId) {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/call_frames/out/debug/call_frames.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/call_frames/out/debug/call_frames-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = CallFramesTestContract::new(id.to_string(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn can_get_contract_id() {
    let (instance, id) = get_call_frames_instance().await;
    let result = instance.methods().get_id().call().await.unwrap();
    assert_eq!(result.value, id);
}

#[tokio::test]
async fn can_get_code_size() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.methods().get_code_size().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_first_param() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.methods().get_first_param().call().await.unwrap();
    // Hash the function name with Sha256
    let mut hasher = Sha256::new();
    let function_name = "get_first_param()";
    hasher.update(function_name);
    let function_name_hash = hasher.finalize();
    // Grab the first 4 bytes of the hash per https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#function-selector-encoding
    let function_name_hash = &function_name_hash[0..4];
    // Convert the bytes to decimal value
    let selector = function_name_hash[3] as u64
        + 256
            * (function_name_hash[2] as u64
                + 256 * (function_name_hash[1] as u64 + 256 * function_name_hash[0] as u64));
    assert_eq!(result.value, selector);
}

#[tokio::test]
async fn can_get_second_param_u64() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance
        .methods()
        .get_second_param_u64(101)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 101);
}

#[tokio::test]
async fn can_get_second_param_bool() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance
        .methods()
        .get_second_param_bool(true)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, true);
}

#[tokio::test]
async fn can_get_second_param_struct() {
    let (instance, _id) = get_call_frames_instance().await;
    let expected = TestStruct {
        value_0: 42,
        value_1: true,
    };
    let result = instance
        .methods()
        .get_second_param_struct(expected.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, expected);
}

#[tokio::test]
async fn can_get_second_param_multiple_params() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance
        .methods()
        .get_second_param_multiple_params(true, 42)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, (true, 42));
}

#[tokio::test]
async fn can_get_second_param_multiple_params2() {
    let (instance, _id) = get_call_frames_instance().await;
    let expected_struct = TestStruct {
        value_0: 42,
        value_1: true,
    };
    let expected_struct2 = TestStruct2 { value: 100 };
    let result = instance
        .methods()
        .get_second_param_multiple_params2(300, expected_struct.clone(), expected_struct2.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, (300, expected_struct, expected_struct2));
}

fn is_within_range(n: u64) -> bool {
    if n <= 0 || n > VM_MAX_RAM {
        false
    } else {
        true
    }
}
