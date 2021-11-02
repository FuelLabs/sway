library auth_testing_abi;

abi AuthTesting {
  fn returns_gm_one(gas: u64, coins: u64, color: b256, input: ()) -> bool;
}
