contract;

mod ownable;
use ownable::*;

storage {
    owner: b256 = b256::zero(),
    data: u64 = 0,
}

abi MyAbi : Ownable {
    #[storage(read, write)]
    fn set_data_if_owner(new_value: u64);
}

impl StorageHelpers for Contract {
    #[storage(read)]
    fn get_owner() -> b256 {
        storage.owner.read()
    }

    #[storage(write)]
    fn set_owner(owner: b256) {
        storage.owner.write(owner)
    }
}

impl Ownable for Contract { }

impl MyAbi for Contract {
    #[storage(read, write)]
    fn set_data_if_owner(new_value: u64) {
        Self::only_owner();
        storage.data.write(new_value);
    }
}
