contract;

use std::{context::call_frames::{first_param, second_param}, address::Address, token::mint_to_address};

abi Asset {
    fn mint_and_send_to_address(amount: u64, recipient: Address) -> (u64, (u64, Address));
}

impl Asset for Contract {
    // Returns the function selector and arguments for testing purposes
    fn mint_and_send_to_address(amount: u64, recipient: Address) -> (u64, (u64, Address)) {
        mint_to_address(amount, recipient);
        (first_param(), second_param::<(u64, Address)>())
    }
}