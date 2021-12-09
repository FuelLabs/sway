library context_testing_abi;

abi ContextTesting {
  fn get_id(gas: u64, coins: u64, color: b256, input: ()) -> b256;
  fn get_amount(gas: u64, coins: u64, color: b256, input: ()) -> u64;
  fn get_token_id(gas: u64, coins: u64, color: b256, input: ()) -> b256;
  fn get_gas(gas: u64, coins: u64, color: b256, input: ()) -> u64;
  fn get_global_gas(gas: u64, coins: u64, color: b256, input: ()) -> u64;
}

