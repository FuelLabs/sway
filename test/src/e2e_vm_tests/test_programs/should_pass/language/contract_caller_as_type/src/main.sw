script;

const ADDRESS = 0x1234123412341234123412341234123412341234123412341234123412341234;
const OTHER_ADDRESS = 0x2234123412341234123412341234123412341234123412341234123412341234;



fn main() -> u64 {
  let caller: ContractCaller<SomeAbi> = contract_caller(SomeAbi, ADDRESS);
  let caller_2 = contract_caller(OtherAbi, OTHER_ADDRESS);
  return 42;
}

abi OtherAbi {
  fn foo() -> u64;
  fn bar() -> u64;
}

abi SomeAbi {
  fn foo() -> u64;
  fn bar() -> u64;
}

fn contract_caller(abi_name: ContractCaller<_>, address: b256) -> ContractCaller<_> {
  let caller = abi(abi_name, ADDRESS);
  caller
}
