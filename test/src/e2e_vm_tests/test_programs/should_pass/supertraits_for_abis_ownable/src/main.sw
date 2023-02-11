contract;

pub struct OwnershipTransferred {
    previous_owner: b256,
    new_owner: b256,
}

storage {
    owner: b256 = std::constants::ZERO_B256,
    data: u64 = 0,
}

trait Ownable {
    // No methods in the interface. The user shouldn't need to implement anything manually.
} {
    // These are all default-implemented ABI methods

    // All storage references to "owner" have been substituted with the storage variable itself
    // because we currently do not support storage refs

    #[storage(read)]
    fn owner() -> b256 {
        storage.owner
    }

    #[storage(read)]
    fn only_owner() {
        assert(std::auth::msg_sender().unwrap() == Identity::Address(Address::from(storage.owner)));
    }

    #[storage(write)]
    fn renounce_ownership() {
        storage.owner = std::constants::ZERO_B256;
    }

    #[storage(read, write)]
    fn transfer_ownership(new_owner: b256) {
        assert(new_owner != std::constants::ZERO_B256);
        let old_owner = storage.owner;
        storage.owner = new_owner;
        std::logging::log(OwnershipTransferred {
            previous_owner: old_owner,
            new_owner: new_owner,
        });
    }
}

abi MyAbi : Ownable {
    #[storage(read, write)]
    fn set_data_if_owner(new_value: u64);
}

impl Ownable for Contract { }

impl MyAbi for Contract {
    #[storage(read, write)]
    fn set_data_if_owner(new_value: u64) {
        Self::only_owner();
        storage.data = new_value
    }
}
