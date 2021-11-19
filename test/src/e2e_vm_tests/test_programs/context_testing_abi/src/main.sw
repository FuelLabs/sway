library context_testing_abi;

abi ContextTesting {
  fn get_id(gas: u64, coins: u64, color: b256, input: ()) -> b256;
  fn get_value(gas: u64, coins: u64, color: b256, input: ()) -> u64;
  fn get_color(gas: u64, coins: u64, color: b256, input: ()) -> b256;
  fn get_gas(gas: u64, coins: u64, color: b256, input: ()) -> u64;
  fn get_global_gas(gas: u64, coins: u64, color: b256, input: ()) -> u64;
}

