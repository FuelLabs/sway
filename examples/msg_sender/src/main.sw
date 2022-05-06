contract;

use std::{address::Address, assert::assert, chain::auth::{AuthError, Sender, msg_sender}, panic::panic, result::*};

abi MyOwnedContract {
    fn receive(field_1: u64) -> bool;
}

const OWNER: b256 = 0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c;

impl MyOwnedContract for Contract {
    fn receive(field_1: u64) -> bool {
        let sender: Result<Sender, AuthError> = msg_sender();
        if let Sender::Address(addr) = sender.unwrap() {
            assert(addr.into() == OWNER);
        } else {
            panic(0);
        };

        true
    }
}
