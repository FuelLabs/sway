script;
use auth_testing_abi::AuthTesting;
use std::assert::assert;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, 0x8c65dd66e3d56a405b5cb329ade3a36e961f4e23038fa3bb3d066feebbf39c1f);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
