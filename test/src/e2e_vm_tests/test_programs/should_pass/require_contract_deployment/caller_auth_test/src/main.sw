script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0xb7832750b7213edfd4e4c5c3cf0f2e2b5407d588a80e2ecf4bddb1ef84963c01);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
