
script;


fn main(addr: b256) -> u64 {
  let caller: ContractCaller<SomeAbi> = abi(SomeAbi, addr);
  let _ = caller.baz();
  return 42;
}

abi SomeAbi {
  fn baz() -> u32;
  fn quux() -> u64;
}

