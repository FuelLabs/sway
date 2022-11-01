contract;

use std::{chain::auth::*, context::{call_frames::contract_id, gas}, reentrancy::*};

use reentrancy_attacker_abi::Attacker;
use reentrancy_target_abi::Target;

// Return the sender as a ContractId or panic:
fn get_msg_sender_id_or_panic(result: Result<Identity, AuthError>) -> ContractId {
    match result {
        Result::Ok(s) => {
            match s {
                Identity::ContractId(v) => v,
                _ => revert(0),
            }
        },
        _ => {
            revert(0);
        },
    }
}

impl Target for Contract {
    fn reentrancy_detected() -> bool {
        if is_reentrant() {
            true
        } else {
            let result: Result<Identity, AuthError> = msg_sender();
            let id = get_msg_sender_id_or_panic(result);
            let id = id.value;
            let caller = abi(Attacker, id);

            // this call transfers control to the attacker contract, allowing it to execute arbitrary code.
            let return_value = caller.evil_callback_1();
            false
        }
    }

    fn reentrance_denied() {
        // panic if reentrancy detected
        reentrancy_guard();

        let result: Result<Identity, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        // this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        let return_value = caller.evil_callback_2();
    }

    fn cross_function_reentrance_denied() {
        // panic if reentrancy detected
        reentrancy_guard();

        let result: Result<Identity, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);
        let id = id.value;
        let caller = abi(Attacker, id);

        // this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        let return_value = caller.evil_callback_3();
    }

    fn intra_contract_call() {
        let this = abi(Target, ContractId::into(contract_id()));
        this.cross_function_reentrance_denied();
    }

    fn guarded_function_is_callable() -> bool {
        // panic if reentrancy detected
        reentrancy_guard();
        true
    }
}
