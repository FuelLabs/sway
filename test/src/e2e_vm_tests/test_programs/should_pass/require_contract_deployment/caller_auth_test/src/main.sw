script;
use auth_testing_abi::AuthTesting;
use std::assert::assert;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, 0xac3198df9174e06cef4bc55ebfc006c1bd8d9958d701a8916e062b7459800a6e);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
