contract;

struct Adder {
    _vals: (u64, u64),
}

abi Fork {
    #[storage(read)]
    fn get_count() -> u64;

    #[storage(read, write)]
    fn increment_count(adder: Adder);
}

abi ForkCaller {
    fn call_increment_count(contract_id: ContractId, adder: Adder) -> u64;
    fn check_current_count(contract_id: ContractId) -> u64;
}

impl ForkCaller for Contract {
    fn check_current_count(contract_id: ContractId) -> u64 {
        let fork = abi(Fork, contract_id.bits());
        fork.get_count()
    }

    fn call_increment_count(contract_id: ContractId, adder: Adder) -> u64 {
        let fork = abi(Fork, contract_id.bits());
        fork.increment_count(adder);
        fork.get_count()
    }
}
