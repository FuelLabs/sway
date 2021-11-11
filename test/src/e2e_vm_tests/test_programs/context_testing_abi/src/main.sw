library context_testing_abi;

abi ContextTesting {
  fn returns_id(gas: u64, coins: u64, color: b256, input: ()) -> b256;
  fn returns_value(gas: u64, coins: u64, color: b256, input: ()) -> u64;
  fn returns_token_id(gas: u64, coins: u64, color: b256, input: ()) -> b256;
}

