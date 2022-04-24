use fuel_tx::Salt;
use fuel_vm::consts::VM_MAX_RAM;
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, parameters::TxParameters};
use fuels_signers::util::test_helpers;

abigen!(
    TestRegistersContract,
    "test_projects/registers/out/debug/registers-abi.json",
);

// Compile contract, create node and deploy contract, returning TestRegistersContract contract instance
// TO DO :
//    -  Ability to return any type of Contract.
//    -  Return a result
async fn deploy_test_registers_instance() -> TestRegistersContract {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    TestRegistersContract::new(id.to_string(), provider, wallet)
}

#[tokio::test]
async fn can_get_overflow() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_overflow().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_program_counter() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_program_counter().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_start_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_stack_start_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_stack_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_frame_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_frame_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_heap_ptr() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_heap_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_error() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_error().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_global_gas() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_global_gas().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_context_gas() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_context_gas().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_balance() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_balance().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_instrs_start() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_instrs_start().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_return_value() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_return_value().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_return_length() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_return_length().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_flags() {
    let instance = deploy_test_registers_instance().await;
    let result = instance.get_flags().call().await.unwrap();
    assert_eq!(result.value, 0);
}

fn is_within_range(n: u64) -> bool {
    if n <= 0 || n > VM_MAX_RAM {
        false
    } else {
        true
    }
}
