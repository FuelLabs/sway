// ANCHOR: full_wallet
contract;

use std::{
    chain::auth::{
        AuthError,
        msg_sender,
    },
    constants::BASE_ASSET_ID,
    context::{
        call_frames::msg_asset_id,
        msg_amount,
    },
    token::transfer_to_address,
};

// ANCHOR: abi_import
use wallet_abi::Wallet;
// ANCHOR_END: abi_import
const OWNER_ADDRESS = Address::from(0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);

storage {
    balance: u64 = 0,
}

// ANCHOR: abi_impl
impl Wallet for Contract {
    #[storage(read, write)]
    fn receive_funds() {
        if msg_asset_id() == BASE_ASSET_ID {
            // If we received `BASE_ASSET_ID` then keep track of the balance.
            // Otherwise, we're receiving other native assets and don't care
            // about our balance of tokens.
            storage.balance += msg_amount();
        }
    }

    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient_address: Address) {
        // Note: The return type of `msg_sender()` can be inferred by the
        // compiler. It is shown here for explicitness.
        let sender: Result<Identity, AuthError> = msg_sender();
        match sender.unwrap() {
            Identity::Address(addr) => assert(addr == OWNER_ADDRESS),
            _ => revert(0),
        };

        let current_balance = storage.balance;
        assert(current_balance >= amount_to_send);

        storage.balance = current_balance - amount_to_send;

        // Note: `transfer_to_address()` is not a call and thus not an
        // interaction. Regardless, this code conforms to
        // checks-effects-interactions to avoid re-entrancy.
        transfer_to_address(amount_to_send, BASE_ASSET_ID, recipient_address);
    }
}
// ANCHOR_END: abi_impl
// ANCHOR_END: full_wallet
