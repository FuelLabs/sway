script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, 0x7fc20db4f2c7c4b1ea38808449a438c36117b462d593a29c41c19b0be31f64e8);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
