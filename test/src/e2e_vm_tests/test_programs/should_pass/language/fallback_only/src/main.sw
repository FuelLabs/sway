contract;

use std::execution::run_external;

#[namespace(SRC1822)]
storage {
    target: ContractId = ContractId::zero(),
}

#[fallback]
#[storage(read)]
fn fallback() {
    run_external(storage.target.read())
}
