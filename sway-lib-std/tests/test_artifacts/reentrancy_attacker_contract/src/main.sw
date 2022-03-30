contract;

use std::contract_id::ContractId;
use reentrancy_target_abi::Target;
use std::chain::auth::*;
use std::context::call_frames::contract_id;
use std::constants::NATIVE_ASSET_ID;
use reentrancy_attacker_abi::Attacker;
use std::result::*;
use std::panic::panic;

impl Attacker for Contract {
    fn launch_attack(target: ContractId) -> bool {
        let id = target.value;
        let caller = abi(Target, id);
        caller.reentrancy_detected()
    }

    fn launch_thwarted_attack(target: ContractId) {
        let id = target.value;
        let caller = abi(Target, id);
        caller.reentrance_denied();
    }

    fn innocent_call(target: ContractId) -> bool {
        let id = target.value;
        let caller = abi(Target, id);
        let return_val = caller.guarded_function();
        return_val
    }

    fn evil_callback(some_value: u64) -> bool {
        let mut success = false;
        let result: Result<Sender, AuthError> = msg_sender();
        let attacker_caller = abi(Attacker, ~ContractId::into(contract_id()));

        let target_id = if let Sender::ContractId(v) = result.unwrap() {
            v
        } else {
            ~ContractId::from(NATIVE_ASSET_ID)
        };

        let reentrancy_detected = attacker_caller.launch_attack(target_id);

        if !reentrancy_detected {
            success
        } else {
            success = true;
            success
        }
    }

    fn innocent_callback(){}
}
