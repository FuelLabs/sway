script;
use std::{chain::auth::caller_is_external, constants::ETH_ID};
use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
  let caller = abi(AuthTesting, 0x27829e78404b18c037b15bfba5110c613a83ea22c718c8b51596e17c9cb1cd6f);

  caller.returns_gm_one(1000, 0, ETH_ID, ())
}
