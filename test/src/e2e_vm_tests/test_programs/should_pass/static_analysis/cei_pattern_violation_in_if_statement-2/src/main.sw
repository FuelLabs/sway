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
        // interaction in the condition, effect in a branch
        if {
            // interaction
            abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae)
                .deposit_quads(amount);
            true
        } {
            // effect -- therefore violation of CEI where effect should go before interaction
            write_quads(
                0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae,
                0,
                (),
            )
        }
    }

    #[storage(write)]
    fn deposit_slot(amount: u64) {
        // interaction in the condition, effect in a branch
        if {
            // interaction
            abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae)
                .deposit_slot(amount);
            true
        } {
            // effect -- therefore violation of CEI where effect should go before interaction
            write_slot(
                0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae,
                (),
            )
        }
    }
}
