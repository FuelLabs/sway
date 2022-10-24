script;
use auth_testing_abi::AuthTesting;
use std::assert::assert;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0xbd5727c9cdd8ae457f94a99cbba11966b50374f3d12c2f4649dd63fdb674361a);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
