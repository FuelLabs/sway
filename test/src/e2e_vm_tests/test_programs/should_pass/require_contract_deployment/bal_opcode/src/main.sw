script;

use std::{assert::assert, constants::NATIVE_ASSET_ID, contract_id::ContractId};
use balance_test_abi::BalanceTest;

fn main() -> bool {
    // @todo switch to using ContractId when abi signature changes.
    let balance_test_contract_id = 0x11dc3309952fa0f6d65abf1f57bc1b7fafca29459a8050d6eb44bce2241c2aa0;

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
