script;

use std::constants::ETH_ID;
use std::chain::assert;
use std::contract_id::ContractId;
use balance_test_abi::BalanceTest;


fn main() -> bool{
    // @todo switch to using ContractId when abi signature changes.
    let balance_test_contract_id = 0x99c656acab368e78e4014dd5b7d4501d32a0511e5f18ce908c64a217ba3b0d02;

    let balance_test_contract = abi(BalanceTest, balance_test_contract_id);
    let number = balance_test_contract.get_42(1000, 0, ETH_ID, ());

    let balance = asm(token_bal, token: ETH_ID, id: balance_test_contract_id) {
        bal token_bal token id;
        token_bal: u64
    };
    assert(balance == 0);
    assert(number == 42);

    true
}
