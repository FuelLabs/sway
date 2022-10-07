contract;

use std::storage::StorageMap;

struct Data {
    value: u64,
}

storage {
    value: StorageMap<u64, Option<Data>> = StorageMap {},
}

abi MyContract {
    #[storage(read)]
    fn is_none() -> bool;
}

impl MyContract for Contract {
    #[storage(read)]
    fn is_none() -> bool {
        storage.value.get(0).is_none()
    }
}
