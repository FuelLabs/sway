library balance_test_abi;

abi BalanceTest {
  fn get_42(gas: u64, coins: u64, color: b256, input: ()) -> u64;
}