script;

use auth_testing_abi::AuthTesting;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x66d9f99ddeeff7d1c6d3b986afd5d20029860289cb74c64e30c255730966d24f;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const CONTRACT_ID = 0x21ec5dab0f00c7718d752037fbb491280f14c124f903450618a641d53f5b88d3;
=======
const CONTRACT_ID = 0x8859754199cfd28bc850cf5061ef29cbd9c238e53b6f0a6861347f278a498cdd;
>>>>>>> 5a1a9d79c (updating contract ids)

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, CONTRACT_ID);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
