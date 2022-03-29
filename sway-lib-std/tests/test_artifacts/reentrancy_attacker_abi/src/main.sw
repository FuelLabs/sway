library reentrancy_attacker_abi;

use std::contract_id::ContractId;

abi Attacker {
    fn launch_attack(target: ContractId) -> bool;
    fn launch_thwarted_attack(target: ContractId) -> bool;
    fn innocent_callback(some_value: u64) -> bool;
}
