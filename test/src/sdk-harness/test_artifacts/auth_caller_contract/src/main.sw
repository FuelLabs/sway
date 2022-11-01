contract;

use auth_testing_abi::*;
use std::chain::auth::*;

abi AuthCaller {
    fn call_auth_contract(auth_id: ContractId, expected_id: ContractId) -> bool;
}

impl AuthCaller for Contract {
    // TODO: improve this to return the ContractId itself.
    // This is a workaround for the MissingData("cannot parse custom type with no components") error
    fn call_auth_contract(auth_id: ContractId, expected_id: ContractId) -> bool {
        let auth_contract = abi(AuthTesting, ContractId::into(auth_id));
        auth_contract.returns_msg_sender(expected_id)
    }
}
