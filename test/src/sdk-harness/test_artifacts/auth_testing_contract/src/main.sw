contract;

use std::address::Address;
use std::chain::auth::*;
use std::contract_id::ContractId;
use auth_testing_abi::*;
use std::result::*;
use std::assert::assert;
use std::identity::Identity;

impl AuthTesting for Contract {
    fn is_caller_external() -> bool {
        caller_is_external()
    }

    fn returns_msg_sender(expected_id: ContractId) -> bool {
        let result: Result<Identity, AuthError> = msg_sender();
        let mut ret = false;
        if result.is_err() {
            ret = false;
        }
        let unwrapped = result.unwrap();
        match unwrapped {
            Identity::ContractId(v) => {ret = true},
            _ => {ret = false},
        }
        ret
    }

    fn returns_msg_sender_address(expected_id: Address) -> bool {
        let result: Result<Identity, AuthError> = msg_sender();
        let mut ret = false;
        if result.is_err() {
            ret = false;
        }
        let unwrapped = result.unwrap();
        match unwrapped {
            Identity::Address(v) => {ret = true},
            _ => {ret = false},
        }
        ret
    }
}
