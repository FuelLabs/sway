script;

use std::contract_id::*;
use std::abi::*;


abi SomeAbi {
  fn foo() -> u64;
  fn bar() -> u64;
}


fn main() -> bool {
    let id = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);

    let caller : ContractCaller<SomeAbi> = abi::contract_at(SomeAbi, id);

    true
}
