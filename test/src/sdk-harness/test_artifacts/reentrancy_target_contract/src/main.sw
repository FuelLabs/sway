contract;

use std::{assert::assert, chain::auth::*, constants::ZERO, context::{call_frames::contract_id, gas}, contract_id::ContractId, panic::panic, reentrancy::*, result::*};
use reentrancy_attacker_abi::Attacker;
use reentrancy_target_abi::Target;

// Return the sender as a ContractId or panic:
fn get_msg_sender_id_or_panic(result: Result<Sender, AuthError>) -> ContractId {
    if let Result::Ok(s) = result {
        if let Sender::ContractId(v) = s {
            v
        } else {
            panic(0);
        }
    } else {
        panic(0);
    }
}

impl Target for Contract {
    fn reentrancy_detected() -> bool {
        if is_reentrant() {
            true
        } else {
            let result: Result<Sender, AuthError> = msg_sender();
            let id = get_msg_sender_id_or_panic(result);
            let id = id.value;
            let caller = abi(Attacker, id);

            /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
            let return_value = caller.evil_callback_1();
            false
        }
    }

    fn reentrance_denied() {
        // panic if reentrancy detected
        reentrancy_guard();

        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        let return_value = caller.evil_callback_2();
    }

    fn cross_function_reentrance_denied() {
        // panic if reentrancy detected
        reentrancy_guard();

        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        let return_value = caller.evil_callback_3();
    }

    fn intra_contract_call() {
        let this = abi(Target, ~ContractId::into(contract_id()));
        this.cross_function_reentrance_denied();
    }

    fn guarded_function_is_callable() -> bool {
        // panic if reentrancy detected
        reentrancy_guard();
        true
    }
}
