contract;

use std::storage::store;

abi TestAbi {
  #[storage(write)]
  fn deposit(amount: u64);
}

fn pure_function(x: u64) -> u64 {
  x
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit(amount: u64) {
    // the function argument is a code block with CEI pattern violation
    pure_function(
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit(amount);
        // effect -- therefore violation of CEI where effect should go before interaction
        store(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
        42
      }
    );
  }
}
