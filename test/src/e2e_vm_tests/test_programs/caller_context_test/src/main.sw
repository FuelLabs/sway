script;

use std::context::Context;
use auth_testing_abi::AuthTesting;
use std::constants::ETH_COLOR;

// set up a simple contract
// get the id of that contract
// make a simple getter function which returns `Context.id()`
// compare the 2 ids

// send coins to the contract
//

fn main() -> bool {
    let caller = abi(ContextTesting, <Address>);
    caller.returns_id(1000, 0, ETH_COLOR, ())
}