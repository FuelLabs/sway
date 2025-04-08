use fuel_vm::consts::VM_MAX_RAM;
use fuels::prelude::*;

abigen!(Contract(
    name = "TestRegistersContract",
    abi = "test_projects/registers/out/release/registers-abi.json",
));

// Compile contract, create node and deploy contract, returning TestRegistersContract contract instance
// TO DO :
//    -  Ability to return any type of Contract.
//    -  Return a result
async fn deploy_test_registers_instance() -> TestRegistersContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/registers/out/release/registers.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestRegistersContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn can_get_overflow() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_overflow().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_program_counter() {
    let instance = deploy_test_registers_instance().await;
    let result = instance
        .methods()
        .get_program_counter()
        .call()
        .await
        .unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_start_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance
        .methods()
        .get_stack_start_ptr()
        .call()
        .await
        .unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_stack_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_frame_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_frame_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_heap_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_heap_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_error() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_error().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_global_gas() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_global_gas().call().await.unwrap();
    assert_ne!(result.value, 0);
}

#[tokio::test]
async fn can_get_context_gas() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_context_gas().call().await.unwrap();
    assert_ne!(result.value, 0);
}

#[tokio::test]
async fn can_get_balance() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_balance().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_instrs_start() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_instrs_start().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_return_value() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_return_value().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_return_length() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_return_length().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_flags() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.methods().get_flags().call().await.unwrap();
    assert_eq!(result.value, 0);
}

fn is_within_range(n: u64) -> bool {
    n > 0 && n <= VM_MAX_RAM
}
