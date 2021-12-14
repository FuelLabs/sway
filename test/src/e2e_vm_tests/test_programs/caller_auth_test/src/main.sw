script;
use std::chain::auth::caller_is_external;
use auth_testing_abi::AuthTesting;
use std::constants::ETH_ID;

// should be false in the case of a script
fn main() -> bool {
  let caller = abi(AuthTesting, 0x573a352216d15ffc712e048b640a3d1ad1b0c16a674adfb4dee0c2fcacf0298b);

  caller.returns_gm_one(1000, 0, ETH_ID, ())
}
