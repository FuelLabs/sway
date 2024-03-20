script;

use auth_testing_abi::AuthTesting;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x66d9f99ddeeff7d1c6d3b986afd5d20029860289cb74c64e30c255730966d24f;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x105954328a6682ec269dc73fe74198a4c5311775aec8b809e754000b3524bd0d;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, CONTRACT_ID);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
