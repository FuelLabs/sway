contract;

// ANCHOR: identity
storage {
    owner: Option<Identity> = None,
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
        assert(storage.owner.read().is_none() || storage.owner.read().unwrap() == msg_sender().unwrap());
        storage.owner.write(owner);
    }

    #[storage(read)]
    fn action() {
        assert(storage.owner.read().unwrap() == msg_sender().unwrap());
        // code
    }
}
// ANCHOR_END: implementation
