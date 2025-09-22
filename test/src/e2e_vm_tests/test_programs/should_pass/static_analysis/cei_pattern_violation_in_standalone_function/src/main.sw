contract;

use std::storage::storage_api::write;

abi TestAbi {
    #[storage(write)]
    fn deposit();
}

#[storage(write)]
fn standalone_function<const N: u64>() {
    let other_contract = abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);

    let _ = __dbg(N);

    // interaction
    other_contract.deposit();
    // effect -- therefore violation of CEI where effect should go before interaction
    let storage_key = 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae;
    write(storage_key, 0, ());
}

impl TestAbi for Contract {
    #[storage(write)]
    fn deposit() {
      standalone_function::<5>();
    }
}
