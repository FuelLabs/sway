use fuel_tx::ContractId;
use fuels::prelude::*;
use fuels::test_helpers;
use fuels_abigen_macro::abigen;
use sha2::{Sha256, Digest};

abigen!(
    HashingTestContract,
    "test_projects/hashing/out/debug/hashing-abi.json"
);

fn hash_u64(number: u64) -> [u8; 32] {
    // Note!
    // Numbers will be padded into u64 in sway regardless of whether you declare a smaller type
    // Therefore tests pass because we use a rust u64 type rather than any smaller type
    Sha256::digest(number.to_be_bytes()).into()
}

fn hash_bool(value: bool) -> [u8; 32] {
    let hash = if value { Sha256::digest([0, 0, 0, 0, 0, 0, 0, 1]) } else { Sha256::digest([0, 0, 0, 0, 0, 0, 0, 0]) };
    hash.into()
}

async fn get_hashing_instance() -> (HashingTestContract, ContractId, LocalWallet, LocalWallet) {
    let compiled =
        Contract::load_sway_contract("test_projects/hashing/out/debug/hashing.bin").unwrap();

    // Hacky way to get 2 addresses
    let (provider, wallet1) = test_helpers::setup_test_provider_and_wallet().await;
    let (_, wallet2) = test_helpers::setup_test_provider_and_wallet().await;
    
    let id = Contract::deploy(&compiled, &provider, &wallet1, TxParameters::default())
        .await
        .unwrap();
    let instance = HashingTestContract::new(id.to_string(), provider, wallet1.clone());

    (instance, id, wallet1, wallet2)
}

#[tokio::test]
async fn test_hash_u64() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    // Check that hashing the same `u64` results in the same hash
    let sha256_result1 = instance.get_s256_hash_u64(42).call().await.unwrap();
    let sha256_result2 = instance.get_s256_hash_u64(42).call().await.unwrap();
    assert_eq!(sha256_result1.value, sha256_result2.value);

    let keccak256_result1 = instance.get_k256_hash_u64(42).call().await.unwrap();
    let keccak256_result2 = instance.get_k256_hash_u64(42).call().await.unwrap();
    assert_eq!(keccak256_result1.value, keccak256_result2.value);
}

#[tokio::test]
async fn test_sha256_u8() {
    let (instance, _id, _, _) = get_hashing_instance().await;

    let expected_1 = hash_u64(254);
    let expected_2 = hash_u64(253);
    
    let call_1 = instance.sha256_u8(254u8).call().await.unwrap();
    let call_2 = instance.sha256_u8(254u8).call().await.unwrap();
    let call_3 = instance.sha256_u8(253u8).call().await.unwrap();

    assert_eq!(call_1.value, call_2.value);
    assert_ne!(call_1.value, call_3.value);

    assert_eq!(expected_1, call_1.value);
    assert_eq!(expected_2, call_3.value);
}

#[tokio::test]
async fn test_sha256_u16() {
    let (instance, _id, _, _) = get_hashing_instance().await;

    let expected_1 = hash_u64(65534);
    let expected_2 = hash_u64(65533);

    let call_1 = instance.sha256_u16(65534u16).call().await.unwrap();
    let call_2 = instance.sha256_u16(65534u16).call().await.unwrap();
    let call_3 = instance.sha256_u16(65533u16).call().await.unwrap();

    assert_eq!(call_1.value, call_2.value);
    assert_ne!(call_1.value, call_3.value);

    assert_eq!(expected_1, call_1.value);
    assert_eq!(expected_2, call_3.value);
}

#[tokio::test]
async fn test_sha256_u32() {
    let (instance, _id, _, _) = get_hashing_instance().await;

    let expected_1 = hash_u64(4294967294);
    let expected_2 = hash_u64(4294967293);

    let call_1 = instance.sha256_u32(4294967294u32).call().await.unwrap();
    let call_2 = instance.sha256_u32(4294967294u32).call().await.unwrap();
    let call_3 = instance.sha256_u32(4294967293u32).call().await.unwrap();

    assert_eq!(call_1.value, call_2.value);
    assert_ne!(call_1.value, call_3.value);

    assert_eq!(expected_1, call_1.value);
    assert_eq!(expected_2, call_3.value);
}

#[tokio::test]
async fn test_sha256_u64() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    
    let expected_1 = hash_u64(18446744073709551613);
    let expected_2 = hash_u64(18446744073709551612);

    let call_1 = instance.sha256_u64(18446744073709551613).call().await.unwrap();
    let call_2 = instance.sha256_u64(18446744073709551613).call().await.unwrap();
    let call_3 = instance.sha256_u64(18446744073709551612).call().await.unwrap();

    assert_eq!(call_1.value, call_2.value);
    assert_ne!(call_1.value, call_3.value);

    assert_eq!(expected_1, call_1.value);
    assert_eq!(expected_2, call_3.value);
}

#[tokio::test]
async fn test_sha256_str() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_str(String::from("John")).call().await.unwrap();
    let result2 = instance.sha256_str(String::from("John")).call().await.unwrap();
    let result3 = instance.sha256_str(String::from("Nick")).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_bool() {
    let (instance, _id, _, _) = get_hashing_instance().await;

    let expected_1 = hash_bool(true);
    let expected_2 = hash_bool(false);

    let call_1 = instance.sha256_bool(true).call().await.unwrap();
    let call_2 = instance.sha256_bool(true).call().await.unwrap();
    let call_3 = instance.sha256_bool(false).call().await.unwrap();
    
    assert_eq!(call_1.value, call_2.value);
    assert_ne!(call_1.value, call_3.value);

    assert_eq!(expected_1, call_1.value);
    assert_eq!(expected_2, call_3.value);
}

#[tokio::test]
async fn test_sha256_b256() {
    let (instance, _id, wallet1, wallet2) = get_hashing_instance().await;
    let address1 = wallet1.address();
    let address2 = wallet2.address();

    let result1 = instance.sha256_b256(*address1).call().await.unwrap();
    let result2 = instance.sha256_b256(*address1).call().await.unwrap();
    let result3 = instance.sha256_b256(*address2).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_tuple() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_tuple((true, 5)).call().await.unwrap();
    let result2 = instance.sha256_tuple((true, 5)).call().await.unwrap();
    let result3 = instance.sha256_tuple((true, 6)).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_array() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_array(1, 5).call().await.unwrap();
    let result2 = instance.sha256_array(1, 5).call().await.unwrap();
    let result3 = instance.sha256_array(1, 6).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_struct() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_struct(true).call().await.unwrap();
    let result2 = instance.sha256_struct(true).call().await.unwrap();
    let result3 = instance.sha256_struct(false).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_enum() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_enum(true).call().await.unwrap();
    let result2 = instance.sha256_enum(true).call().await.unwrap();
    let result3 = instance.sha256_enum(false).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}