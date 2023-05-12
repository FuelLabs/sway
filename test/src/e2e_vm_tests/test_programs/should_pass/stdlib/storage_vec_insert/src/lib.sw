contract;

use std::storage::storage_vec::*;

abi MyContract {
    #[storage(read, write)]
    fn test_function() -> bool;
}

storage {
    foo: StorageVec<u32> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn test_function() -> bool {
        storage.foo.push(0);
        storage.foo.insert(0, 123);
        true
    }
}

#[test]
fn test_test_function() {
    let caller = abi(MyContract, CONTRACT_ID);
    let res: bool = caller.test_function();
}