/*#[tokio::test]
#[tokio::test]
async fn can_get_b256() {
    let (instance, id) = get_contract_instance().await;
    let n: [u8; 32] = id.into();
    let result = instance.get_b256().call().await.unwrap();
    assert_eq!(result.value, n);
}*/

use fuels::prelude::*;
use fuels_abigen_macro::abigen;

abigen!(
    TestStorageContract,
    "test_projects/storage/out/debug/storage-abi.json",
);

async fn get_test_storage_instance() -> TestStorageContract {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/storage/out/debug/storage.bin",
        &wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    TestStorageContract::new(id.to_string(), wallet)
}

#[tokio::test]
async fn can_store_and_get_bool() {
    let instance = get_test_storage_instance().await;
    let b = true;
    instance.store_bool(b).call().await.unwrap();
    let result = instance.get_bool().call().await.unwrap();
    assert_eq!(result.value, b);
}

#[tokio::test]
async fn can_store_and_get_u8() {
    let instance = get_test_storage_instance().await;
    let n = 8;
    instance.store_u8(n).call().await.unwrap();
    let result = instance.get_u8().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_and_get_u16() {
    let instance = get_test_storage_instance().await;
    let n = 16;
    instance.store_u16(n).call().await.unwrap();
    let result = instance.get_u16().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_and_get_u32() {
    let instance = get_test_storage_instance().await;
    let n = 32;
    instance.store_u32(n).call().await.unwrap();
    let result = instance.get_u32().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_and_get_u64() {
    let instance = get_test_storage_instance().await;
    let n = 64;
    instance.store_u64(n).call().await.unwrap();
    let result = instance.get_u64().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_b256() {
    let instance = get_test_storage_instance().await;
    let n: [u8; 32] = [2; 32];
    instance.store_b256(n).call().await.unwrap();
    let result = instance.get_b256().call().await.unwrap();
    assert_eq!(result.value, n);
}

#[tokio::test]
async fn can_store_small_struct() {
    let instance = get_test_storage_instance().await;
    let s = SmallStruct {
        x: 42,
    };
    instance.store_small_struct(s.clone()).call().await.unwrap();
    let result = instance.get_small_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_medium_struct() {
    let instance = get_test_storage_instance().await;
    let s = MediumStruct {
        x: 42,
        y: 66,
    };
    instance.store_medium_struct(s.clone()).call().await.unwrap();
    let result = instance.get_medium_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_large_struct() {
    let instance = get_test_storage_instance().await;
    let s = LargeStruct {
        x: 13,
        y: [6; 32],
        z: 77,
    };
    instance.store_large_struct(s.clone()).call().await.unwrap();
    let result = instance.get_large_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

#[tokio::test]
async fn can_store_very_large_struct() {
    let instance = get_test_storage_instance().await;
    let s = 
        VeryLargeStruct {
            x: 42,
            y: [9; 32],
            z: [7; 32],
            w: LargeStruct {
                x: 13,
                y: [6; 32],
                z: 77,
            }
    };
    instance.store_very_large_struct(s.clone()).call().await.unwrap();
    let result = instance.get_very_large_struct().call().await.unwrap();
    assert_eq!(result.value, s);
}

/*#[tokio::test]
async fn can_get_overflow() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_overflow().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_program_counter() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_program_counter().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_start_ptr() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_stack_start_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_stack_ptr() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_stack_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_frame_ptr() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_frame_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_heap_ptr() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_heap_ptr().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_error() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_error().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_global_gas() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_global_gas().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_context_gas() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_context_gas().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_balance() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_balance().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_instrs_start() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_instrs_start().call().await.unwrap();
    assert!(is_within_range(result.value));
}

#[tokio::test]
async fn can_get_return_value() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_return_value().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_return_length() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_return_length().call().await.unwrap();
    assert_eq!(result.value, 0);
}

#[tokio::test]
async fn can_get_flags() {
    let instance = get_test_storage_instance().await;
    let result = instance.get_flags().call().await.unwrap();
    assert_eq!(result.value, 0);
}

fn is_within_range(n: u64) -> bool {
    if n <= 0 || n > VM_MAX_RAM {
        false
    } else {
        true
    }
}*/
