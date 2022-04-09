contract;

use balance_test_abi::BalanceTest;

impl BalanceTest for Contract {
    fn get_42() -> u64 {
        42
    }
}
