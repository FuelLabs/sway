script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0x10f04ba40bd185d6e2e326a9f8be6d1c1f96b7a021faecea1bd46fc4b5cce885);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
