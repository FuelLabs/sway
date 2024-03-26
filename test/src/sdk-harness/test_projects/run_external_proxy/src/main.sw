contract;

use std::execution::run_external;
use std::constants::ZERO_B256;

configurable {
    TARGET: ContractId = ContractId::from(ZERO_B256)
}

abi RunExternalTest{
    fn double_value(foo: u64) -> u64;
    fn does_not_exist_in_the_target(foo: u64) -> u64;
}


impl RunExternalTest for Contract {
    fn double_value(_foo: u64) -> u64 {
        run_external(TARGET)
    }

    fn does_not_exist_in_the_target(_foo: u64) -> u64 {
        run_external(TARGET)
    }
}
