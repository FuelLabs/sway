contract;

use std::storage::storage_vec::*;
use std::hash::*;
use std::identity::*;

abi MyContract {
    #[storage(read, write)]
    fn withdraw();
}

storage {
    balances: StorageMap<Identity, u64> = StorageMap::<Identity, u64> {},
    vec: StorageVec<u64> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn withdraw() {
        let sender = msg_sender().unwrap();
        let bal = storage.balances.get(sender).try_read().unwrap_or(0);

        assert(bal > 0);

        let caller = abi(MyContract, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);
        caller.withdraw();

        // should only report storage write after external contract call
        // should _not_ report storage read after external contract call
        storage.balances.insert(sender, 0);
        // should only report storage write after external contract call
        // should _not_ report storage read after external contract call
        storage.vec.clear();
    }
}
