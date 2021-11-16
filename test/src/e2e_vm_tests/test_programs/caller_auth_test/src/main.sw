script;
use std::chain::auth::caller_is_external;
use auth_testing_abi::AuthTesting;
use std::constants::ETH_COLOR;

// should be false in the case of a script
fn main() -> bool {
  let caller = abi(AuthTesting, 0x8ca92c2a448e86f374657604a3d62f3d83226f86acfed38e8124cce826926f7f);

  caller.returns_gm_one(1000, 0, ETH_COLOR, ())
}
