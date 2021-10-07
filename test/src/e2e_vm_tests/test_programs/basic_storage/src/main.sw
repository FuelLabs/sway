contract;
use std::storage::*;
use std::hash::*;
use basic_storage_abi::*;

impl StoreU64 for Contract {
  fn store_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, storage: StoreU64Request) {
   let storage_key = hash_u64(storage.key, HashMethod::Sha256);
   store_u64(storage_key, storage.value);
  }

  fn get_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, key: u64) -> u64 {
    let storage_key = hash_u64(key, HashMethod::Sha256);
    let blah = get_u64(storage_key);
    return 42;
  }
}

