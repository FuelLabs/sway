script;

use std::constants::BASE_ASSET_ID;
use balance_test_abi::BalanceTest;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xe50966cd6b1da8fe006e3e876e08f3df6948ce426e1a7cfe49fba411b0a11f89;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xb0b0589ced70b31fb34cbb7fbb1b0e4046cc61c2ffe79cdb06a617bf24d9458c;

fn main() -> bool {
    let balance_test_contract = abi(BalanceTest, CONTRACT_ID);
    let number = balance_test_contract.get_42 {
        gas: u64::max()
    }
    ();

    let balance = asm(asset_bal, asset: BASE_ASSET_ID, id: CONTRACT_ID) {
        bal asset_bal asset id;
        asset_bal: u64
    };
    assert(balance == 0);
    assert(number == 42);

    true
}
