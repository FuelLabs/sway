use fuel_tx::ContractId;
use fuels::prelude::*;
use fuels::test_helpers;
use fuels_abigen_macro::abigen;

abigen!(
    HashingTestContract,
    "test_projects/hashing/out/debug/hashing-abi.json"
);

enum Location {
    Earth,
    Mars,
}

struct Person {
    name: String,
    age: u8,
    birth_place: Location,
    stats: Stats,
    alive: bool,
    random_b256: [[u8; 32]; 1]
}

struct Stats {
    strength: u64,
    agility: u64
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
    let result1 = instance.sha256_u8(254).call().await.unwrap();
    let result2 = instance.sha256_u8(254).call().await.unwrap();
    let result3 = instance.sha256_u8(253).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_u16() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_u16(65534).call().await.unwrap();
    let result2 = instance.sha256_u16(65534).call().await.unwrap();
    let result3 = instance.sha256_u16(65533).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_u32() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_u32(4294967294).call().await.unwrap();
    let result2 = instance.sha256_u32(4294967294).call().await.unwrap();
    let result3 = instance.sha256_u32(4294967293).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_u64() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let result1 = instance.sha256_u64(18446744073709551613).call().await.unwrap();
    let result2 = instance.sha256_u64(18446744073709551613).call().await.unwrap();
    let result3 = instance.sha256_u64(18446744073709551612).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
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
    let result1 = instance.sha256_bool(true).call().await.unwrap();
    let result2 = instance.sha256_bool(true).call().await.unwrap();
    let result3 = instance.sha256_bool(false).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
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

// #[tokio::test]
// async fn test_sha256_array() {
//     let (instance, _id, _, _) = get_hashing_instance().await;
//     // There is a bug in using arrays as parameters
//     let array_1: [u64; 2] = [5, 4];
//     let result1 = instance.sha256_array(array_1.to_vec()).call().await.unwrap();
//     let result2 = instance.sha256_array(array_1.to_vec()).call().await.unwrap();
//     // let result3 = instance.sha256_array([5, 99]).call().await.unwrap();
//     assert_eq!(result1.value, result2.value);
//     // assert_ne!(result1.value, result3.value);
// }

#[tokio::test]
async fn test_sha256_struct() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let person1 = Person {
        name: String::from("John"),
        age: 18,
        birth_place: Location::Earth,
        stats: Stats {
            strength: 5,
            agility: 8
        },
        alive: true,
        random_b256: [[1; 32]]
    };

    let person2 = Person {
        name: String::from("Nick"),
        age: 18,
        birth_place: Location::Earth,
        stats: Stats {
            strength: 5,
            agility: 8
        },
        alive: true,
        random_b256: [[1; 32]]
    };

    let result1 = instance.sha256_struct(person1).call().await.unwrap();
    let result2 = instance.sha256_struct(person1).call().await.unwrap();
    let result3 = instance.sha256_struct(person2).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}

#[tokio::test]
async fn test_sha256_enum() {
    let (instance, _id, _, _) = get_hashing_instance().await;
    let location1 = Location::Earth;
    let location2 = Location::Mars;
    let result1 = instance.sha256_enum(location1).call().await.unwrap();
    let result2 = instance.sha256_enum(location1).call().await.unwrap();
    let result3 = instance.sha256_enum(location2).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
    assert_ne!(result1.value, result3.value);
}