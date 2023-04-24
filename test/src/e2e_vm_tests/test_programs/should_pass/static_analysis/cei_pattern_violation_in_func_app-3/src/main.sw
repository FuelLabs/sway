contract;

use std::storage::storage_api::write;

abi TestAbi {
  #[storage(write)]
  fn deposit(amount: u64);
}

#[storage(write)]
fn do_something(_x: u64) {
  // effect
  write(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, 0, ());
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
