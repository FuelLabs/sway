contract;

// ANCHOR: abi
abi Wallet {
    #[storage(read, write)]
    fn receive();

    #[storage(read, write)]
    fn send(amount: u64, recipient: Identity);
}
// ANCHOR_END: abi
// ANCHOR: implementation
use std::{
    auth::msg_sender,
    call_frames::msg_asset_id,
    constants::BASE_ASSET_ID,
    context::msg_amount,
    token::transfer,
};

storage {
    balance: u64 = 0,
}

const OWNER = Address::from(0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);

impl Wallet for Contract {
    #[storage(read, write)]
    fn receive() {
        assert(msg_asset_id() == BASE_ASSET_ID);
        storage.balance += msg_amount();
    }

    #[storage(read, write)]
    fn send(amount: u64, recipient: Identity) {
        assert(msg_sender().unwrap() == Identity::Address(OWNER));
        storage.balance -= amount;
        transfer(amount, BASE_ASSET_ID, recipient);
    }
}
// ANCHOR: implementation
