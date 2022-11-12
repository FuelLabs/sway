contract;

use std::storage::store;

abi TestAbi {
  #[storage(write)]
  fn deposit(amount: u64);
}

#[storage(write)]
fn do_something(x: u64) {
  // effect
  store(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit(amount: u64) {
    // function's argument is a code block with interaction, function does storage write
    do_something(
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit(amount);
        42
      },
    );
  }
}
