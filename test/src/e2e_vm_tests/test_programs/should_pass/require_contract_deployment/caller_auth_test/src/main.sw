script;

use auth_testing_abi::AuthTesting;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID: b256 = 0xc2eec20491b53aab7232cbd27c31d15417b4e9daf0b89c74cc242ef1295f681f;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID: b256 = 0x6ab2528386b7d41c88a6dabe8a05b2f7b392984d4ec9b90be9e71a4f8815e1a8; // AUTO-CONTRACT-ID ../../test_contracts/auth_testing_contract --release

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, CONTRACT_ID);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
