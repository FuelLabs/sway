library;

pub struct OwnershipTransferred {
    previous_owner: b256,
    new_owner: b256,
}

pub trait StorageHelpers {
    #[storage(read)]
    fn get_owner() -> b256;

    #[storage(write)]
    fn set_owner(owner: b256);
}

pub trait Ownable : StorageHelpers {
    // No methods in the interface. The user shouldn't need to implement anything manually.
} {
    // These are all default-implemented ABI methods

    #[storage(read)]
    fn owner() -> b256 {
        Self::get_owner()
    }

    #[storage(read)]
    fn only_owner() {
        assert(msg_sender().unwrap() == Identity::Address(Address::from(Self::get_owner())));
    }

    #[storage(write)]
    fn renounce_ownership() {
        Self::set_owner(std::constants::ZERO_B256);
    }

    #[storage(read, write)]
    fn transfer_ownership(new_owner: b256) {
        assert(new_owner != std::constants::ZERO_B256);
        let old_owner = Self::get_owner();
        Self::set_owner(new_owner);
        std::logging::log(OwnershipTransferred {
            previous_owner: old_owner,
            new_owner: new_owner,
        });
    }
}
