script;

use balance_test_abi::BalanceTest;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xf6cd545152ac83225e8e7df2efb5c6fa6e37bc9b9e977b5ea8103d28668925df;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x49460391f59d838a381add127bd3bc47a840c18e78d55df0d332159dfb01acfa; // AUTO-CONTRACT-ID ../../test_contracts/balance_test_contract --release

fn main() -> bool {
    let balance_test_contract = abi(BalanceTest, CONTRACT_ID);
    let number = balance_test_contract.get_42 {
        gas: u64::max()
    }
    ();

    let balance = asm(asset_bal, asset: AssetId::base(), id: CONTRACT_ID) {
        bal asset_bal asset id;
        asset_bal: u64
    };
    assert(balance == 0);
    assert(number == 42);

    true
}
