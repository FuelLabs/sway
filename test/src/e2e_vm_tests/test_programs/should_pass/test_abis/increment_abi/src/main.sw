library increment_abi;

abi Incrementor {
    #[storage(read, write)]
    fn increment(initial_value: u64) -> u64;
    #[storage(read)]
    fn get() -> u64;
}
