contract;

use increment_abi::Incrementor;
use std::storage::store_u64;
use std::storage::get_u64;

const key = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl Incrementor for Contract {
  fn initialize(gas: u64, amt: u64, color: b256, initial_value: u64) -> u64 {
    store_u64(key, initial_value);
    initial_value
  }
  fn increment(gas: u64, amt: u64, color: b256, increment_by: u64) -> u64 {
    let new_val = get_u64(key) + 1;
    store_u64(key, new_val);
    new_val
  }
}
