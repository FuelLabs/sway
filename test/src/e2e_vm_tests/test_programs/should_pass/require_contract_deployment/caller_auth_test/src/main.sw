script;

use auth_testing_abi::AuthTesting;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xc2eec20491b53aab7232cbd27c31d15417b4e9daf0b89c74cc242ef1295f681f;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xeb42a6ba3f230441d7ade26f54cfb9f64fe8dd6397cc685fefc7f3d082d4d2ce;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, CONTRACT_ID);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
