library auth_testing_abi;
use std::result::Result;

abi AuthTesting {
  fn returns_gm_one(gas: u64, coins: u64, asset_id: b256, input: ()) -> bool;
  fn returns_msg_sender(gas: u64, coins: u64, asset_id: b256, input: ()) -> Result;
}
