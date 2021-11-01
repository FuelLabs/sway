library increment_abi;

abi Incrementor {
  fn initialize(gas: u64, amt: u64, coin: b256, initial_value: u64) -> u64;
  fn increment(gas: u64, amt: u64, coin: b256, initial_value: u64) -> u64;
}
