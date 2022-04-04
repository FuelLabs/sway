script;

use std::{assert::assert, constants::NATIVE_ASSET_ID, contract_id::ContractId};
use balance_test_abi::BalanceTest;

fn main() -> bool {
    // @todo switch to using ContractId when abi signature changes.
    let balance_test_contract_id = 0xb4c0d8c9056c0cde34b66e7e4e3f361d927d26ffdc16c2645dd0e2699bc96cad;

    let balance_test_contract = abi(BalanceTest, balance_test_contract_id);
    let number = balance_test_contract.get_42 {
        gas: 1000
    }
    ();

    let balance = asm(token_bal, token: NATIVE_ASSET_ID, id: balance_test_contract_id) {
        bal token_bal token id;
        token_bal: u64
    };
    assert(balance == 0);
    assert(number == 42);

    true
}
