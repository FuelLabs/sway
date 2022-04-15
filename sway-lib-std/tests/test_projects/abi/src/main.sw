script;

use std::contract_id::*;
use std::abi::*;


abi SomeAbi {
  fn foo() -> u64;
}


fn main() -> u64 {
    // Contract deployed with this contract ID implements `foo` (../test_artifacts/abi_wrapper_testing_contract)
    let id = ~ContractId::from(0x49949f60837951e5b19685b5580e4ecf027db4f6fc465ee668751b20df4aeac5);

    // Get contract caller using wrapper and test a call
    let caller : ContractCaller<SomeAbi> = contract_at(SomeAbi, id);
    let result = caller.foo();
    result
}
