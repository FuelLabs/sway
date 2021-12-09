script;

use std::constants::ETH_ID;
use std::chain::assert;
use std::contract_id::ContractId;


fn main() -> bool{
    let test_contract_id = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000007);

    let balance = asm(token_bal, token: ETH_ID, id: test_contract_id) {
        bal token_bal token id;
        token_bal: u64
    };
    assert(balance == 0);

    true
}
