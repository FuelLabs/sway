script;
use std::{chain::{assert, auth::{caller_is_external, AuthError}}, constants::ETH_ID, result::*};
use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
  let caller = abi(AuthTesting, 0x46bd0d4a848314a56f156e4a1f6c118abfd425a877fe5b4212660ba3a6793ff8);

  assert(caller.returns_gm_one(1000, 0, ETH_ID, ()));
  let res: Result = caller.returns_msg_sender(1000, 0, ETH_ID, ());
  assert(~Result::is_err(res));
  // TODO: impl Eq for Result so we can test more precisely:
  // assert(res == Result::Err(AuthError::ContextError));

  true
}
