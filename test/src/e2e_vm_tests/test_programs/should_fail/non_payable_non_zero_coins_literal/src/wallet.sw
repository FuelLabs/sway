contract;

mod wallet_abi;

use std::{asset::transfer, auth::AuthError, call_frames::msg_asset_id, context::msg_amount};

use wallet_abi::Wallet;
const OWNER_ADDRESS = Address::from(0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);

storage {
    balance: u64 = 0,
}

impl Wallet for Contract {
    #[payable, storage(read, write)]
    fn receive_funds() {
        if msg_asset_id() == AssetId::base() {
            storage.balance += msg_amount();
        }
    }

    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient_address: Address) {
        let sender: Result<Identity, AuthError> = msg_sender();
        match sender.unwrap() {
            Identity::Address(addr) => assert(addr == OWNER_ADDRESS),
            _ => revert(0),
        };

        let current_balance = storage.balance;
        assert(current_balance >= amount_to_send);

        storage.balance = current_balance - amount_to_send;

        transfer(
            Identity::Address(recipient_address),
            AssetId::base(),
            amount_to_send,
        );
    }
}
