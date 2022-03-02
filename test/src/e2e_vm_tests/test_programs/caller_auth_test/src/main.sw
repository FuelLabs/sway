script;
use std::{chain::auth::caller_is_external, constants::ETH_ID};
use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, 0xbf65a4702b9614db428964fc034aa21b39eeb5bffe8c0a518cc2e013730a978f);

    caller.returns_gm_one {
        gas: 1000, coins: 0, asset_id: ETH_ID
    }
    ()
}
