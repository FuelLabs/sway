library increment_abi;

abi Incrementor {
    fn initialize(initial_value: u64) -> u64;
    fn increment(initial_value: u64) -> u64;
    fn get() -> u64;
}
