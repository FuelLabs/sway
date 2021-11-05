library context_testing_abi;

abi ContextTesting {
  fn returns_id(gas: u64, coins: u64, color: b256, input: ()) -> b256;
}

