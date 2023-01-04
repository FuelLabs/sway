script;

dep test_cases;

use std::revert::require;

pub enum TestCaseError {
    U64: (),
    B256: (),
}


fn main(target: ContractId) {

    //require(test_cases::test_u64(target), TestCaseError::U64);
    require(test_cases::test_b256(target), TestCaseError::B256);

}
