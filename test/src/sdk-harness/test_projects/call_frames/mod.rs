use fuel_vm::consts::VM_MAX_RAM;
use fuels::{accounts::wallet::WalletUnlocked, prelude::*, types::ContractId};

abigen!(Contract(
    name = "CallFramesTestContract",
    abi = "test_projects/call_frames/out/release/call_frames-abi.json"
));

async fn get_call_frames_instance() -> (CallFramesTestContract<WalletUnlocked>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/call_frames/out/release/call_frames.bin",
        LoadConfiguration::default(),
    )
    .unwrap();

    let id = id.deploy(&wallet, TxPolicies::default()).await.unwrap();
    let instance = CallFramesTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn can_get_id_contract_id_this() {
    let (instance, id) = get_call_frames_instance().await;
    let result = instance
        .methods()
        .get_id_contract_id_this()
        .call()
        .await
        .unwrap();
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
    assert_eq!(result.value, 10480);
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
    assert_eq!(result.value, 10508);
}

#[tokio::test]
async fn can_get_second_param_bool() {
    let (instance, _id) = get_call_frames_instance().await;
    let result = instance.methods().get_second_param_bool(true);
    let result = result.call().await.unwrap();
    assert!(result.value);
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
    n > 0 && n <= VM_MAX_RAM
}
