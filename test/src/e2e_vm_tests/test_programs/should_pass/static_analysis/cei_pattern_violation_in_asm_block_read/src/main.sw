contract;

const KEY: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

abi TestAbi {
    #[storage(read)]
    fn deposit() -> u64;
}

impl TestAbi for Contract {
    #[storage(read)]
    fn deposit() -> u64{
        let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

        // interaction
        other_contract.deposit();
        // effect -- therefore violation of CEI where effect should go before interaction
        asm(key: KEY, is_set, res) {
            srw res is_set key;
            res: u64
        }
    }
}
