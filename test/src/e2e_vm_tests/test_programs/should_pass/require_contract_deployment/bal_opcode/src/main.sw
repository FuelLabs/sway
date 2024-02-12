script;

use std::constants::BASE_ASSET_ID;
use balance_test_abi::BalanceTest;

fn main() -> bool {
    // @todo switch to using ContractId when abi signature changes.
    let balance_test_contract_id = 0x56c6624d1b117cdf4e319385257206ed6a8011dac4f6c4d6523da33918f0d78c;

    let balance_test_contract = abi(BalanceTest, balance_test_contract_id);
    let number = balance_test_contract.get_42 {
        gas: u64::max()
    }
    ();

    let balance = asm(asset_bal, asset: BASE_ASSET_ID, id: balance_test_contract_id) {
        bal asset_bal asset id;
        asset_bal: u64
    };
    assert(balance == 0);
    assert(number == 42);

    true
}
