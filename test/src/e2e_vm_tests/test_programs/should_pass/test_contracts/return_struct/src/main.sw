contract;

pub mod data_structures;
mod interface;

use interface::MyContract;
use data_structures::MyStruct;
use std::hash::*;

storage {
    a: StorageMap<u64, MyStruct> = StorageMap::<u64, MyStruct> {}
}

impl MyContract for Contract {
    #[storage(read)]
    fn test_function() -> Option<MyStruct> {
        storage.a.get(1).try_read()
    }
}

#[test]
fn collect_my_contract_gas_usages() {
    let caller = abi(MyContract, CONTRACT_ID);
    let _ = caller.test_function();
}
