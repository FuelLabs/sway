script;
use std::{chain::auth::caller_is_external, constants::ETH_ID};
use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, 0x4bc450bf26a5ebca955ed8e58ca281bcba64065a802a2b1cfa5cdefdeec1610e);

    caller.returns_gm_one {
        gas: 1000, coins: 0, asset_id: ETH_ID
    }
    ()
}
