contract;

use std::{address::Address, token::mint_to_address};

abi Asset {
    fn mint_and_send_to_address(amount: u64, recipient: Address) -> bool;
}

impl Asset for Contract {
    fn mint_and_send_to_address(amount: u64, recipient: Address) -> bool {
        mint_to_address(amount, recipient);
        true
    }
}
