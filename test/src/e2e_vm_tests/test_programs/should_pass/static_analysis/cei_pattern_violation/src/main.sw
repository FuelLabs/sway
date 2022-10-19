contract;

use std::storage::store;

// TODO when the test harness can expect warnings, _this should warn about CEI_!

abi TestAbi {
  #[storage(write)]
  fn deposit(gas: u64, amount: u64, color: b256,  unused: ());
}

#[storage(write)]
fn deposit(gas: u64,  amount: u64, color: b256, unused: ()) {
let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

// interaction
other_contract.deposit(gas, amount, color, unused);
// [toplevel function] effect -- therefore violation of CEI where effect should go before interaction
let storage_key = 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae;
//store(storage_key, ());
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit(gas: u64,  amount: u64, color: b256, unused: ()) {
    let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

    // interaction
    other_contract.deposit(gas, amount, color, unused);
    // [trait/abi method] effect -- therefore violation of CEI where effect should go before interaction
    let storage_key = 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae;
    store(storage_key, ());
  }
}
