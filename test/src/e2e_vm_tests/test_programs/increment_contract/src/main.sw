contract;

use increment_abi::Incrementor;
use std::storage::{store, get};

const key = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl Incrementor for Contract {
  fn initialize(gas: u64, amt: u64, asset_id: b256, initial_value: u64) -> u64 {
    store(key, initial_value);
    initial_value
  }
  fn increment(gas: u64, amt: u64, asset_id: b256, increment_by: u64) -> u64 {
    let new_val = get::<u64>(key) + 1;
    // check that monomorphization doesn't overwrite the type of the above
    let dummy = get::<u32>(key) + 1;
    store(key, new_val);
    new_val
  }
}
