script;

use std::{assert::assert, constants::BASE_ASSET_ID, contract_id::ContractId};
use balance_test_abi::BalanceTest;

fn main() -> bool {
    // @todo switch to using ContractId when abi signature changes.
    let balance_test_contract_id = 0x597e5ddb1a6bec92a96a73e4f0bc6f6e3e7b21f5e03e1c812cd63cffac480463;

    let balance_test_contract = abi(BalanceTest, balance_test_contract_id);
    let number = balance_test_contract.get_42 {
        gas: 1000
    }
    ();

    let balance = asm(token_bal, token: BASE_ASSET_ID, id: balance_test_contract_id) {
        bal token_bal token id;
        token_bal: u64
    };
    assert(balance == 0);
    assert(number == 42);

    true
}
