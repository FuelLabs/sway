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
    // a code block inside the function body: a simpler version of the CEI analysis does not catch this
    {
        let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

        // interaction
        other_contract.deposit_quads(amount);

        // effect -- therefore violation of CEI where effect should go before interaction
        let storage_key = 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae;
        write_quads(storage_key, 0, ());
    }
  }

  #[storage(write)]
  fn deposit_slot(amount: u64) {
    // a code block inside the function body: a simpler version of the CEI analysis does not catch this
    {
        let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

        // interaction
        other_contract.deposit_slot(amount);

        // effect -- therefore violation of CEI where effect should go before interaction
        let storage_key = 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae;
        write_slot(storage_key, ());
    }
  }
}
