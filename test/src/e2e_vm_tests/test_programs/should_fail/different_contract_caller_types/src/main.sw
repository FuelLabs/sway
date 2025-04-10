script;

const ADDRESS: b256 = 0x1234123412341234123412341234123412341234123412341234123412341234;

fn main() -> u64 {
  let caller: ContractCaller<SomeAbi> = contract_caller();
  return 42;
}

abi SomeAbi {
  fn foo() -> u64;
  fn bar() -> u64;
}

abi OtherAbi {
  fn foo() -> u64;
  fn bar() -> u64;
}

// should not allow these two abis to resolve, as OtherAbi != SomeAbi
fn contract_caller() -> ContractCaller<SomeAbi> {
  let caller = abi(OtherAbi, ADDRESS);
  caller
}

