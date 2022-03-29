contract;

use std::reentrancy::is_reentrant;
use std::chain::panic;
use std::contract_id::ContractId;
use std::constants::ETH_ID;
use std::chain::auth::msg_sender;
use std::context::gas;
use attacker_abi::Attacker;
use target_abi::Target;

impl Target for Contract {
    fn can_be_reentered() -> bool {
        let safe_from_reentry: bool = false;
        // let attacker_id = msg_sender();
        // let caller = abi(Attacker, attacker_id);
        // TEMP: use hardcoded attacker ContractID until Result type can be better utilized
        let caller = abi(Attacker, <ATTACKER_ID>);
        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        caller.innocent_callback(42);

        let was_reentered = is_reentrant();
        was_reentered
    }

    fn reentrant_proof() -> bool {
        let mut reentrant_proof = false;
        if is_reentrant() {
            reentrant_proof = true;
        };
        // let attacker_id = msg_sender();
        // let caller = abi(Attacker, attacker_id);
        // TEMP: use hardcoded attacker ContractID until Result type can be better utilized
        let caller = abi(Attacker, <ATTACKER_ID>);
        /// this call transfers control to the attacker contract, allowing it to execute arbitrary code.
        caller.innocent_callback(42);
        reentrant_proof
    }
}