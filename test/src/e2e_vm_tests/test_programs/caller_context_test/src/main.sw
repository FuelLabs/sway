script;

// use std::context::Context;
use std::constants::ETH_COLOR;
use context_testing_abi::ContextTesting;

fn main() -> bool {
    let deployed_id = 0xad6aaaa1d6fd78f91693ee2cc124fd43d25bd1c015b88b675ee43d6b5e140586;
    let caller = abi(ContextTesting, deployed_id);
    let returned_id = caller.returns_id(1000, 0, ETH_COLOR, ());
    returned_id == deployed_id
}