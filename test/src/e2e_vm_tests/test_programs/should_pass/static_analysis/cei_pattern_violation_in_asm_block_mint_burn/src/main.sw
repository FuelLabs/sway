contract;

abi TestAbi {
    fn mint();
    fn burn();
}

const AMOUNT_TO_BURN: u64 = 100;
const ASSET_ID: b256 = b256::zero();

impl TestAbi for Contract {
    fn mint() {
        let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

        // interaction
        other_contract.mint();
        // effect -- therefore violation of CEI where effect should go before interaction
        asm(r1: AMOUNT_TO_BURN, r2: ASSET_ID) {
            mint r1 r2;
        }
    }

    fn burn() {
        let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

        // interaction
        other_contract.mint();
        // effect -- therefore violation of CEI where effect should go before interaction
        asm(r1: AMOUNT_TO_BURN, r2: ASSET_ID) {
            burn r1 r2;
        }
    }
}
