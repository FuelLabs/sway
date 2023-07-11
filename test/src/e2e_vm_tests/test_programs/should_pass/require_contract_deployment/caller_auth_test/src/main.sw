script;

use auth_testing_abi::AuthTesting;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting,  0x7047587b9e9072f210c187d46988c7559e2f37fb3069150c42cd7fda507c09db);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
