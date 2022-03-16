use fuel_core::service::Config;
use fuel_tx::Salt;
use fuel_vm::consts::VM_MAX_RAM;
use fuels_abigen_macro::abigen;
use fuels_contract::contract::Contract;
use fuels_signers::provider::Provider;

abigen!(
    TestFuelCoinContract,
    "test_projects/registers/out/debug/registers-abi.json",
);

#[tokio::test]
async fn can_get_overflow() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_overflow().call().await.unwrap();

    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_program_counter() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_program_counter().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_start_ptr() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_stack_start_ptr().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_ptr() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_stack_ptr().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_frame_ptr() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_frame_ptr().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_heap_ptr() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_heap_ptr().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_error() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_error().call().await.unwrap();

    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_global_gas() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_global_gas().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_context_gas() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_context_gas().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_balance() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_balance().call().await.unwrap();

    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_instrs_start() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_instrs_start().call().await.unwrap();

    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_return_value() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_return_value().call().await.unwrap();

    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_return_length() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let result = instance.get_return_length().call().await.unwrap();

    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_flags() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/registers/out/debug/registers.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

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
