contract;

use std::storage::storage_api::{write_quads, write_slot};

abi TestAbi {
  #[storage(write)]
  fn deposit_quads(amount: u64);
  #[storage(write)]
  fn deposit_slot(amount: u64);
}

#[storage(write)]
fn do_something_quads(_x: u64) {
  // effect
  write_quads(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, 0, ());
}

#[storage(write)]
fn do_something_slot(_x: u64) {
  // effect
  write_slot(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit_quads(amount: u64) {
    // function's argument is a code block with interaction, function does storage write
    do_something_quads(
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit_quads(amount);
        42
      },
    );
  }

  #[storage(write)]
  fn deposit_slot(amount: u64) {
    // function's argument is a code block with interaction, function does storage write
    do_something_slot(
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit_slot(amount);
        42
      },
    );
  }
}
