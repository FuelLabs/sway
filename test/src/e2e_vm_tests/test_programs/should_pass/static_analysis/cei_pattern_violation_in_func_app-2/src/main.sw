contract;

use std::storage::store;

abi TestAbi {
  #[storage(write)]
  fn deposit(amount: u64);
}

fn pure_function(x: u64, y: u64) -> u64 {
  x
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit(amount: u64) {
    // 1st function argument is a code block with interaction
    // 2nd function argument is a code block with effect
    pure_function(
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit(amount);
        42
      },
      {
        // effect -- therefore violation of CEI where effect should go before interaction
        // (assuming left-to-right function argument evaluation)
        store(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
        43
      }
    );
  }
}
