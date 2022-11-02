contract;
use std::auth::caller_is_external;
use auth_testing_abi::AuthTesting;

impl AuthTesting for Contract {
    fn returns_gm_one() -> bool {
        caller_is_external()
    }
}
