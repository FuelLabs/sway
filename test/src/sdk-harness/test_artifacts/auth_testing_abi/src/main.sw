library auth_testing_abi;

use std::chain::auth::*;

abi AuthTesting {
    fn is_caller_external() -> bool;
    fn returns_msg_sender(expected_id: ContractId) -> bool;
    fn returns_msg_sender_address(expected_id: Address) -> bool;
}
