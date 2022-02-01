script;
use std::{chain::auth::caller_is_external, constants::ETH_ID};
use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
  let caller = abi(AuthTesting, 0xf8aa0c04665af0fd65a6ea6a05e42a57ec737d953af70a200a10bc3c0eec4553);

  caller.returns_gm_one(1000, 0, ETH_ID, ())
}
