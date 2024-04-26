contract;

use std::asset::transfer;

abi TestAbi {
    fn deposit();
}

impl TestAbi for Contract {
    fn deposit() {
        let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

        // interaction
        other_contract.deposit();
        // effect -- therefore violation of CEI where effect should go before interaction
        let amount = 10;
        let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
        let asset = AssetId::from(address);
        let pool = ContractId::from(address);
        // `transfer` uses `tr` asm instruction for ContractId
        transfer(Identity::ContractId(pool), asset, amount);
    }
}
