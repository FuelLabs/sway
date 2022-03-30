contract;

use std::reentrancy::is_reentrant;
use std::panic::panic;
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
    // rename to vulnerable_to_reentry()
    fn can_be_reentered() -> bool {
        let mut was_reentered = false;
        let safe_from_reentry: bool = false;
        let result: Result<Sender, AuthError> = msg_sender();

        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        caller.innocent_callback(42);
        was_reentered = is_reentrant();
        was_reentered
    }

    // rename to reentrancy_detected()
    fn reentrant_proof() -> bool {
        let mut reentrancy_detected = false;
        if is_reentrant() {
            // to actually prevent reentrancy in a contract, simply do:
            // assert(!is_reentrant());
            // for testing, we just set reentrant_proof to 'true' to signify that we can at least detect reentrancy, and could easily forbid it.
            return true;
        };

        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        caller.innocent_callback(42);
        reentrancy_detected
    }
}