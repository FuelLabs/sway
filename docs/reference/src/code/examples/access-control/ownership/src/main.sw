contract;

// ANCHOR: identity
use std::auth::msg_sender;

storage {
    owner: Option<Identity> = Option::None,
}
// ANCHOR_END: identity
// ANCHOR: abi
abi Ownership {
    #[storage(read, write)]
    fn set_owner(owner: Option<Identity>);

    #[storage(read)]
    fn action();
}
// ANCHOR_END: abi
// ANCHOR: implementation
impl Ownership for Contract {
    #[storage(read, write)]
    fn set_owner(owner: Option<Identity>) {
        assert(storage.owner.is_none() || storage.owner.unwrap() == msg_sender().unwrap());
        storage.owner = owner;
    }

    #[storage(read)]
    fn action() {
        assert(storage.owner.unwrap() == msg_sender().unwrap());
        // code
    }
}
// ANCHOR_END: implementation
