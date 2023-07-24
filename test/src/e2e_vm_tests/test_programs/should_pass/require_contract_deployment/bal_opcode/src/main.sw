script;

use std::constants::BASE_ASSET_ID;
use balance_test_abi::BalanceTest;

fn main() -> bool {
    // @todo switch to using ContractId when abi signature changes.
    let balance_test_contract_id = 0xd54323a3316a7cfd26a3bab076a0194dfffbcc9a583d152043f5237fa60753ed;

    let balance_test_contract = abi(BalanceTest, balance_test_contract_id);
    let number = balance_test_contract.get_42 {
        gas: u64::max()
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
