use fuel_vm::consts::VM_MAX_RAM;
use fuels::{prelude::*, tx::ContractId};

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
            "test_artifacts/call_frames/out/debug/call_frames-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = CallFramesTestContract::new(id.to_string(), wallet);

    (instance, id)
}

#[tokio::test]
async fn can_get_contract_id() {
    let (instance, id) = get_call_frames_instance().await;
    let result = instance.get_id().call().await.unwrap();
    assert_eq!(result.value, id);
}

#[tokio::test]
async fn can_get_code_size() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.get_code_size().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_first_param() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.get_first_param().call().await.unwrap();
    // The hash of "get_first_param" is 0x35fd8826
    // The first four bytes converted to decimal is 905807910
    assert_eq!(result.value, 905807910);
}

#[tokio::test]
#[ignore]
async fn can_get_second_param() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.get_second_param().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_second_param_u64() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.get_second_param_u64(101).call().await.unwrap();
    assert_eq!(result.value, 101);
}

#[tokio::test]
async fn can_get_second_param_bool() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.get_second_param_bool(true).call().await.unwrap();
    assert_eq!(result.value, true);
}

#[tokio::test]
async fn can_get_second_param_multiple_params() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.get_second_param_multiple_params(true, 42).call().await.unwrap();
    assert_eq!(result.value, (true, 42));
}

fn is_within_range(n: u64) -> bool {
    if n <= 0 || n > VM_MAX_RAM {
        false
    } else {
        true
    }
}
