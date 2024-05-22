contract;

use std::execution::run_external;
use std::constants::ZERO_B256;

#[namespace(SRC1822)]
storage {
    target: ContractId = ContractId::from(ZERO_B256),
}

#[fallback]
#[storage(read)]
fn fallback() {
    run_external(storage.target.read())
}
