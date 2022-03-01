script;

use std::{chain::assert, constants::ETH_ID, contract_id::ContractId};
use balance_test_abi::BalanceTest;

fn main() -> bool {
    // @todo switch to using ContractId when abi signature changes.
    let balance_test_contract_id = 0x6b5677971f7d0e94d76c18f268d8ccffd04b5b3f3bdb2f1da119b76e376dcf04;

    let balance_test_contract = abi(BalanceTest, balance_test_contract_id);
    let number = balance_test_contract.get_42 {
        gas: 1000, coins: 0, asset_id: ETH_ID
    }
    ();

    let balance = asm(token_bal, token: ETH_ID, id: balance_test_contract_id) {
        bal token_bal token id;
        token_bal: u64
    };
    assert(balance == 0);
    assert(number == 42);

    true
}
