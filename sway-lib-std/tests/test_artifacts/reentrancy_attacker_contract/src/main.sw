contract;

use std::contract_id::ContractId;
use target_abi::Target;
use std::chain::auth::msg_sender;
use std::context::contract_id;
use std::constants::ETH_ID;
use attacker_abi::Attacker;


impl Attacker for Contract {
    fn launch_attack(target: ContractId) -> bool {
        let id = target.value;
        let caller = abi(Target, id);
        let result = caller.can_be_reentered();
        result
    }

     fn launch_thwarted_attack(target: ContractId) -> bool {
         let id = target.value;
         let caller = abi(Target, id);
         let result = caller.reentrant_proof();
         result
     }

    fn innocent_callback(some_value: u64) -> bool {
        let attack_thwarted = true;
        let target_id = msg_sender();
        let attacker_caller = abi(Attacker, ~ContractId::into(contract_id()));
        // TODO: fix this to use the 'target_id' returned by mesage_sender()!
        // attacker_caller.launch_attack(1000, 0, ETH_ID, target_id);
        attacker_caller.launch_attack(<TARGET_ID>);
        // consider use of 'if let' here to set value of attack_thwarted conditionally
        attack_thwarted
    }
}
