script;
use auth_testing_abi::AuthTesting;
use std::assert::assert;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0x6868c510e230173e1f788fd7bdba127ffb92b7408d0e7fface1a32d03c004361);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
