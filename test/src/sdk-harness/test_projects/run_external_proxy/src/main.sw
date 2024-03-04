contract;

use std::execution::run_external;

abi RunExternalTest{
    fn foobar(target: ContractId) -> u64;
}

impl RunExternalTest for Contract {
    fn foobar(target: ContractId) -> u64 {
        run_external(target)
    }
}
