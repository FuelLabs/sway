contract;

use std::reentrancy::is_reentrant;
use std::panic::panic;
use std::assert::assert;
// use std::address::Address;
use std::contract_id::ContractId;
use std::constants::ZERO;
use std::chain::auth::*;
use std::result::*;
use std::context::gas;
use reentrancy_attacker_abi::Attacker;
use reentrancy_target_abi::Target;


// Return the sender as an Address or panic:
fn get_msg_sender_id_or_panic(result: Result<Sender, AuthError>) -> ContractId {
    let mut ret = ~ContractId::from(ZERO);
    if result.is_err() {
        panic(0);
    } else {
        let unwrapped = result.unwrap();
        if let Sender::ContractId(v) = unwrapped {
            ret = v;
        } else {
            panic(0);
        };
    };

    ret
}

impl Target for Contract {
    fn reentrance_denied() {
        // panic if reentrancy detected
        assert(!is_reentrant());
        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        caller.innocent_callback(42);
    }

    fn reentrancy_detected() -> bool {
        if is_reentrant() {
            return true;
        };

        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        caller.innocent_callback(42);

        false
    }
}