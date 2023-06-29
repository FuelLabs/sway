script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0x73dc55710d076e3c547492a29faf0fff7f56b5c1593e5404beaedd36d7004841);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
