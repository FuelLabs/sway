contract;
use std::chain::auth::caller_is_external;
use auth_testing_abi::AuthTesting;

impl AuthTesting for Contract {
  fn returns_gm_one(gas: u64, coins: u64, asset_id: b256, input: ()) -> bool {
     caller_is_external()
  }
}
