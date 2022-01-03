library balance_test_abi;

abi BalanceTest {
  fn get_42(gas: u64, coins: u64, asset_id: b256, input: ()) -> u64;
}
