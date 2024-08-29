contract;

use std::execution::run_external;

storage {
    SRC1822 {
        target: ContractId = ContractId::zero(),
    }
}

#[fallback]
#[storage(read)]
fn fallback() {
    run_external(storage::SRC1822.target.read())
}
