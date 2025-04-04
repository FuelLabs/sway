// ANCHOR: full_wallet
contract;

use std::{asset::transfer, call_frames::msg_asset_id, context::msg_amount};

// ANCHOR: abi_import
use wallet_abi::Wallet;
// ANCHOR_END: abi_import
const OWNER_ADDRESS: Address = Address::from(0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);

storage {
    balance: u64 = 0,
}

// ANCHOR: abi_impl
impl Wallet for Contract {
    #[storage(read, write), payable]
    fn receive_funds() {
        if msg_asset_id() == AssetId::base() {
            // If we received the base asset then keep track of the balance.
            // Otherwise, we're receiving other native assets and don't care
            // about our balance of coins.
            storage.balance.write(storage.balance.read() + msg_amount());
        }
    }

    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient_address: Address) {
        let sender = msg_sender().unwrap();
        match sender {
            Identity::Address(addr) => assert(addr == OWNER_ADDRESS),
            _ => revert(0),
        };

        let current_balance = storage.balance.read();
        assert(current_balance >= amount_to_send);

        storage.balance.write(current_balance - amount_to_send);

        // Note: `transfer()` is not a call and thus not an
        // interaction. Regardless, this code conforms to
        // checks-effects-interactions to avoid re-entrancy.
        transfer(
            Identity::Address(recipient_address),
            AssetId::base(),
            amount_to_send,
        );
    }
}
// ANCHOR_END: abi_impl
// ANCHOR_END: full_wallet
