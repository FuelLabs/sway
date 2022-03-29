contract;

use std::reentrancy::is_reentrant;
// use std::panic::panic;
use std::contract_id::ContractId;
use std::constants::NATIVE_ASSET_ID;
use std::chain::auth::*;
use std::result::Result;
use std::context::gas;
use reentrancy_attacker_abi::Attacker;
use reentrancy_target_abi::Target;

fn unwrap_msg_sender(result: Result<Sender, AuthError>) -> ContractId {
    if ! result.is_err() {
        let attacker_id = if let Sender::ContractId(v) = unwrapped {
            v
        } else {
            ~ContractId::from(NATIVE_ASSET_ID)
        };
    };
    attacker_id
}

impl Target for Contract {
    fn can_be_reentered() -> bool {
        let mut was_reentered = false;
        let safe_from_reentry: bool = false;
        let result = msg_sender();
        let id = unwrap_msg_sender(result);

        if id.value != NATIVE_ASSET_ID {
            let val = id.value;
            let caller = abi(Attacker, val);
            /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
            caller.innocent_callback(42);
            was_reentered = is_reentrant();
        };

        was_reentered
    }

    fn reentrant_proof() -> bool {
        let mut reentrant_proof = false;
        if is_reentrant() {
            reentrant_proof = true;
        };
        let result = msg_sender();
        let id = unwrap_msg_sender(result);

        if id.value != NATIVE_ASSET_ID {
            let val = id.value;
            let caller = abi(Attacker, val);
            /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
            caller.innocent_callback(42);
        };
        reentrant_proof
    }
}