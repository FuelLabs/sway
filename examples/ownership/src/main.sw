contract;

use std::constants::ZERO_B256;
use ownership::{*, data_structures::State};

abi OwnershipExample {
    #[storage(write)]
    fn revoke_ownership();
    #[storage(write)]
    fn set_owner(identity: Identity);
    #[storage(read)]
    fn owner() -> State;
    #[storage(read)]
    fn only_owner();
}

// ANCHOR: set_owner_example_storage
storage {
    owner: Ownership = Ownership::initialized(Identity::Address(Address::from(ZERO_B256))),
}
// ANCHOR_END: set_owner_example_storage

impl OwnershipExample for Contract {
    // ANCHOR: revoke_owner_example
    #[storage(write)]
    fn revoke_ownership() {
        storage.owner.renounce_ownership();
    }
    // ANCHOR_END: revoke_owner_example
    // ANCHOR: set_owner_example_function
    #[storage(write)]
    fn set_owner(identity: Identity) {
        storage.owner.set_ownership(identity);
    }
    // ANCHOR_END: set_owner_example_function
    // ANCHOR: get_owner_example
    #[storage(read)]
    fn owner() -> State {
        storage.owner.owner()
    }
    // ANCHOR_END: get_owner_example
    // ANCHOR: only_owner_example
    #[storage(read)]
    fn only_owner() {
        storage.owner.only_owner();
        // Do stuff here
    }
    // ANCHOR_END: only_owner_example
}
