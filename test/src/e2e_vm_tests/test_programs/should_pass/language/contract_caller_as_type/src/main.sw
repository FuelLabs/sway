script;

const ADDRESS = 0x1234123412341234123412341234123412341234123412341234123412341234;

fn main() -> u64 {
  let caller: ContractCaller<SomeAbi> = contract_caller();
  return 42;
}


abi SomeAbi {
  fn foo() -> u64;
  fn bar() -> u64;
}

fn contract_caller() -> ContractCaller<SomeAbi> {
  let caller = abi(SomeAbi, ADDRESS);
}
