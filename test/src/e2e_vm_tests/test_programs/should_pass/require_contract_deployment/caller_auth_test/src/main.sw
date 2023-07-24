script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0x66e88a6499e593af0358dc93f6c0733b783da91caefd58d7bd8579c8a2d0bd1b);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
