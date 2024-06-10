script;

use auth_testing_abi::AuthTesting;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xc2eec20491b53aab7232cbd27c31d15417b4e9daf0b89c74cc242ef1295f681f;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xf2e9d7d515fac2ff33ebdcb1ac306510c4d3b6fc6622524173873f0be31bba08;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, CONTRACT_ID);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
