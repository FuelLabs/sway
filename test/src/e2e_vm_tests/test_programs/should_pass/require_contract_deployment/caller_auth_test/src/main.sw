script;

use auth_testing_abi::AuthTesting;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xd7ef57c654a7e52ee8b85f34c64fa2f8e1a250eceb446cfe9805b175a0a7680f;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xad9e118ed616de791d5b5d82882cb34b069348e1cbcb07d0c7eb166eb10f1715;

// should be false in the case of a script
fn main() -> bool {
    let caller = abi(AuthTesting, CONTRACT_ID);
    let result = caller.returns_gm_one();
    assert(result);
    result
}
