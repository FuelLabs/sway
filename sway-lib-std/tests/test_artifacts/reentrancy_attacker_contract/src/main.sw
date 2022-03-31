contract;

use std::{chain::auth::*, constants::ZERO, context::call_frames::contract_id, contract_id::ContractId, panic::panic, result::*};
use reentrancy_target_abi::Target;
use reentrancy_attacker_abi::Attacker;

// Return the sender as a ContractId or panic:
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

impl Attacker for Contract {
    fn launch_attack(target: ContractId) -> bool {
        let id = target.value;
        let target = abi(Target, id);
        target.reentrancy_detected()
    }

    fn launch_thwarted_attack_1(target: ContractId) {
        let id = target.value;
        let target = abi(Target, id);
        target.reentrance_denied();
    }

    fn launch_thwarted_attack_2(target: ContractId) {
        let id = target.value;
        let target = abi(Target, id);
        target.cross_function_reentrance_denied();
    }

    fn innocent_call(target: ContractId) -> bool {
        let id = target.value;
        let target = abi(Target, id);
        target.guarded_function_is_callable();
        true
    }

    fn evil_callback_1() -> bool {
        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);

        let attacker = abi(Attacker, ~ContractId::into(contract_id()));
        attacker.launch_attack(id)
    }

    fn evil_callback_2() -> bool {
        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);

        let attacker = abi(Attacker, ~ContractId::into(contract_id()));
        attacker.launch_thwarted_attack_1(id);
        true
    }

    fn evil_callback_3() -> bool {
        let result: Result<Sender, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);

        let attacker = abi(Attacker, ~ContractId::into(contract_id()));
        attacker.launch_thwarted_attack_2(id);
        true
    }

    fn innocent_callback() {
    }
}
