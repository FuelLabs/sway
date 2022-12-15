contract;

use std::storage::store;

abi TestAbi {
  #[storage(write)]
  fn deposit(amount: u64);
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit(amount: u64) {
    while true {
      {
        // effect -- violation of CEI where effect should go before interaction
        // this can happen here because this is a loop and the interaction happens
        // at the end of the loop body
        store(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit(amount);
      }
    }
  }
}
