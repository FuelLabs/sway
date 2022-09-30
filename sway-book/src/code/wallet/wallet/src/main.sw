contract;

use interface::Wallet;
use std::{
    chain::auth::msg_sender,
    constants::BASE_ASSET_ID,
    context::{
        call_frames::msg_asset_id,
        msg_amount,
    },
    result::Result,
    token::transfer,
};

storage {
    balance: u64 = 0,
}

impl Wallet for Contract {
    #[storage(read, write)]
    fn receive_funds() {
        if msg_asset_id() == BASE_ASSET_ID {
            // If we received `BASE_ASSET_ID` then keep track of the balance. Otherwise, we're 
            // receiving other native assets and don't care about our balance of tokens.
            storage.balance += msg_amount();
        }
    }

    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient: Identity) {
        assert(msg_sender().unwrap() == Identity::Address(~Address::from(OWNER_ADDRESS)));
        assert(storage.balance >= amount_to_send);

        storage.balance -= amount_to_send;
        // Note: `transfer()` is not a call and thus not an interaction. Regardless, this code 
        // conforms to checks-effects-interactions to avoid re-entrancy.
        transfer(amount_to_send, BASE_ASSET_ID, recipient);
    }
}
