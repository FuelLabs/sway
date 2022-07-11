contract;

use std::{identity::Identity, option::Option};

abi OwnershipExample {
    #[storage(write)]fn revoke_ownership();
    #[storage(write)]fn set_owner(identity: Identity);
    #[storage(read)]fn owner() -> Option<Identity>;
}

storage {
    owner: Option<Identity> = Option::None,
}

impl OwnershipExample for Contract {
    // ANCHOR: revoke_owner_example
    #[storage(write)]fn revoke_ownership() {
        storage.owner = Option::None();
    }
    // ANCHOR_END: revoke_owner_example

    // ANCHOR: set_owner_example
    #[storage(write)]fn set_owner(identity: Identity) {
        storage.owner = Option::Some(identity);
    }
    // ANCHOR_END: set_owner_example

    #[storage(read)]fn owner() -> Option<Identity> {
        storage.owner
    }
}
