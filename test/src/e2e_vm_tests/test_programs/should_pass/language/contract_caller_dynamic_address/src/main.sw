
script;


fn main(addr: b256) -> u64 {
  let caller: ContractCaller<SomeAbi> = abi(SomeAbi, addr);
  // this should revert since we don't have the script data being passed in to the harness
  let _ = caller.baz();
  return 42;
}

abi SomeAbi {
  fn baz() -> u32;
  fn quux() -> u64;
}

