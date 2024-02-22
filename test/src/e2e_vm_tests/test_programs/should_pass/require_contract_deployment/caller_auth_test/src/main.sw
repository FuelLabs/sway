script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, 0x66d9f99ddeeff7d1c6d3b986afd5d20029860289cb74c64e30c255730966d24f);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
