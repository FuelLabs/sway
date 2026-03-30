contract;

use std::storage::storage_api::{write_quads, write_slot};

abi TestAbi {
  #[storage(write)]
  fn deposit_quads(amount: u64);
  #[storage(write)]
  fn deposit_slot(amount: u64);
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit_quads(amount: u64) {
    // 1st tuple component is a code block with interaction
    // 2nd tuple component is a code block with effect
    let _pair: (u64, u64) =
      (
        {
          // interaction
          abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit_quads(amount);
          42
        },
        {
          // effect -- therefore violation of CEI where effect should go before interaction
          // (assuming left-to-right tuple component evaluation)
          write_quads(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, 0, ());
          43
        }
      );
  }

  #[storage(write)]
  fn deposit_slot(amount: u64) {
    // 1st tuple component is a code block with interaction
    // 2nd tuple component is a code block with effect
    let _pair: (u64, u64) =
      (
        {
          // interaction
          abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit_slot(amount);
          42
        },
        {
          // effect -- therefore violation of CEI where effect should go before interaction
          // (assuming left-to-right tuple component evaluation)
          write_slot(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
          43
        }
      );
  }
}
