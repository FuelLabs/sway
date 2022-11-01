contract;

use std::{chain::auth::*, context::call_frames::contract_id};

use reentrancy_target_abi::Target;
use reentrancy_attacker_abi::Attacker;

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

impl Attacker for Contract {
    fn launch_attack(target: ContractId) -> bool {
        let id = target.value;
        let target = abi(Target, id);
        target.reentrancy_detected()
    }

    fn launch_thwarted_attack_1(target: ContractId) {
        let id = target.value;
        let target = abi(Target, id);
        target.intra_contract_call();
    }

    fn launch_thwarted_attack_2(target: ContractId) {
        let id = target.value;
        let target = abi(Target, id);
        target.cross_function_reentrance_denied();
    }

    fn innocent_call(target: ContractId) -> bool {
        let id = target.value;
        let target = abi(Target, id);
        target.guarded_function_is_callable()
    }

    fn evil_callback_1() -> bool {
        let result: Result<Identity, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);

        let attacker = abi(Attacker, ContractId::into(contract_id()));
        attacker.launch_attack(id)
    }

    fn evil_callback_2() -> bool {
        let result: Result<Identity, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);

        let attacker = abi(Attacker, ContractId::into(contract_id()));
        attacker.launch_thwarted_attack_1(id);
        true
    }

    fn evil_callback_3() -> bool {
        let result: Result<Identity, AuthError> = msg_sender();
        let id = get_msg_sender_id_or_panic(result);

        let attacker = abi(Attacker, ContractId::into(contract_id()));
        attacker.launch_thwarted_attack_1(id);
        true
    }

    fn innocent_callback() {}
}
