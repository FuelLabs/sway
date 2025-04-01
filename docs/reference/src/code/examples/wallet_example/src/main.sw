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
    call_frames::msg_asset_id,
    context::msg_amount,
    asset::transfer,
};

storage {
    balance: u64 = 0,
}

const OWNER: Address = Address::from(0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);

impl Wallet for Contract {
    #[storage(read, write)]
    fn receive() {
        assert(msg_asset_id() == AssetId::base());
        storage.balance.write(storage.balance.read() + msg_amount());
    }

    #[storage(read, write)]
    fn send(amount: u64, recipient: Identity) {
        assert(msg_sender().unwrap() == Identity::Address(OWNER));
        storage.balance.write(storage.balance.read() - amount);
        transfer(recipient, AssetId::base(), amount);
    }
}
// ANCHOR: implementation
