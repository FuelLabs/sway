script;

const ADDRESS = 0x1234123412341234123412341234123412341234123412341234123412341234;


fn main() -> u64 {
  let caller: ContractCaller<SomeAbi> = contract_caller::<SomeAbi>(ADDRESS);
  let caller_2 = contract_caller::<SomeAbi>(ADDRESS);
  return 42;
}

abi SomeAbi {
  fn foo() -> u64;
  fn bar() -> u64;
}

fn contract_caller(address: b256, abi_name: Abi) -> ContractCaller<_> {
  let caller = abi(abi_name, ADDRESS);
  caller
}
