library;

abi Incrementor {
    #[storage(read, write)]
    fn increment(initial_value: u64) -> u64;

    #[storage(read, write)]
    fn increment_or_not(initial_value: Option<u64>) -> u64;

    #[storage(read)]
    fn get() -> u64;
}
