contract;

use std::execution::run_external;

abi Proxy {
    #[storage(write)]
    fn set_target_contract(id: ContractId);

    #[storage(read)]
    fn double_input(_value: u64) -> u64;
}

// ANCHOR: proxy
#[namespace(my_storage_namespace)]
storage {
    target_contract: Option<ContractId> = None,
}

impl Proxy for Contract {
    #[storage(write)]
    fn set_target_contract(id: ContractId) {
        storage.target_contract.write(Some(id));
    }

    #[storage(read)]
    fn double_input(_value: u64) -> u64 {
        let target = storage.target_contract.read().unwrap();
        run_external(target)
    }
}
// ANCHOR_END: proxy
