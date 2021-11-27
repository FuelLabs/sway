contract;
use std::storage::*;
use std::hash::*;
use basic_storage_abi::*;

impl StoreU64 for Contract {
  fn store_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, storage: StoreU64Request) {
   store(storage.key, storage.value);
  }

  fn get_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, storage_key: b256) -> u64 {
    get(storage_key)
  }
}

