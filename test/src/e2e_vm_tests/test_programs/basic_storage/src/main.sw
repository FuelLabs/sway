contract;
use std::storage::*;
use std::hash::*;
use basic_storage_abi::*;

impl StoreU64 for Contract {
  fn store_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, storage: StoreU64Request) {
   store_u64(storage.key, storage.value);
  }

  fn get_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, storage_key: b256) -> u64 {
    asm(r1: 88888) {
      log r1 r1 r1 r1;
    };
    get_u64(storage_key)
  }
}

