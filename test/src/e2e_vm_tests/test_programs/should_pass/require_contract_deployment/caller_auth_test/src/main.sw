script;

use auth_testing_abi::AuthTesting;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xc2eec20491b53aab7232cbd27c31d15417b4e9daf0b89c74cc242ef1295f681f;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x507e5e609fdb2a085da8d681519b6bb6b738f2beb0321600c1e3497db3114a37; // AUTO-CONTRACT-ID ../../test_contracts/auth_testing_contract --release

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, CONTRACT_ID);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
