contract;

use std::storage::store;

abi TestAbi {
    #[storage(write)]
    fn deposit(amount: u64);
}

struct S {
    field1: u64,
    field2: u64,
}

impl TestAbi for Contract {
    #[storage(write)]
    fn deposit(amount: u64) {
    // 1st tuple component is a code block with interaction
    // 2nd tuple component is a code block with effect
        let s = S {
            field1:  {
                // interaction
                abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit(amount);
                42
            },
            field2:  {
                // effect -- therefore violation of CEI where effect should go before interaction
                // (assuming left-to-right struct fields evaluation)
                store(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
                43
            },
        };
    }
}
