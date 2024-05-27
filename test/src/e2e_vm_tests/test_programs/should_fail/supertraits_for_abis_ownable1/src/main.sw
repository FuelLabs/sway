contract;

mod ownable;
use ownable::{Ownable, StorageHelpers};

storage {
    owner: b256 = b256::zero(),
    data: u64 = 0,
}

abi MyAbi : Ownable {
    #[storage(read, write)]
    fn set_data_if_owner(new_value: u64);
}

// error: no impl StorageHelpers for Contract

impl Ownable for Contract { }

impl MyAbi for Contract {
    #[storage(read, write)]
    fn set_data_if_owner(new_value: u64) {
        Self::only_owner();
        storage.data = new_value
    }
}
