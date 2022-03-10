contract;

abi BalanceTest {
    fn get_42() -> u64;
}

impl BalanceTest for Contract {
    fn get_42() -> u64 {
        42
    }
}
