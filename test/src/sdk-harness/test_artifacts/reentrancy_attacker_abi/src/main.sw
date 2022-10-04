library reentrancy_attacker_abi;

abi Attacker {
    fn launch_attack(target: ContractId) -> bool;
    fn launch_thwarted_attack_1(target: ContractId);
    fn launch_thwarted_attack_2(target: ContractId);
    fn innocent_call(target: ContractId) -> bool;
    fn evil_callback_1() -> bool;
    fn evil_callback_2() -> bool;
    fn evil_callback_3() -> bool;
    fn innocent_callback();
}
