script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0x98c8c9232f0a2b2558889628ffb36cfc1a7224c9afc104522bfdb1b49a882489);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
