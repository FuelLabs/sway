contract;

use std::address::Address;
use std::chain::auth::*;
use std::contract_id::ContractId;
use auth_testing_abi::*;
use std::result::*;
use std::assert::assert;

impl AuthTesting for Contract {
    fn is_caller_external() -> bool {
        caller_is_external()
    }

    fn returns_msg_sender(expected_id: ContractId) -> bool {
        let result: Result<Sender, AuthError> = msg_sender();
        let mut ret = false;
        if result.is_err() {
            ret = false;
        } else {
            let unwrapped = result.unwrap();
            if let Sender::ContractId(v) = unwrapped {
                assert(v == expected_id);
                ret = true;
            } else {
                ret = false;
            };
        };

        ret
    }

    fn returns_msg_sender_address(expected_id: Address) -> bool {
        let result: Result<Sender, AuthError> = msg_sender();
        let mut ret = false;
        if result.is_err() {
            ret = false;
        } else {
            let unwrapped = result.unwrap();
            if let Sender::Address(v) = unwrapped {
                assert(v == expected_id);
                ret = true;
            } else {
                ret = false;
            }
        };

        ret
    }
}
