contract;

use std::execution::run_external;

configurable {
    TARGET: ContractId = ContractId::zero(),
}

abi RunExternalTest {
    fn double_value(foo: u64) -> u64;
    fn large_value() -> b256;
    fn does_not_exist_in_the_target(foo: u64) -> u64;
}

impl RunExternalTest for Contract {
    fn double_value(_foo: u64) -> u64 {
        __log(1);
        run_external(TARGET)
    }

    fn large_value() -> b256 {
        run_external(TARGET)
    }

    // ANCHOR: does_not_exist_in_the_target
    fn does_not_exist_in_the_target(_foo: u64) -> u64 {
        run_external(TARGET)
    }
    // ANCHOR_END: does_not_exist_in_the_target
}
