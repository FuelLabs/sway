contract;

abi Implementation {
    #[storage(read, write)]
    fn double_input(value: u64) -> u64;
}

// ANCHOR: target
storage {
    value: u64 = 0,
}

impl Implementation for Contract {
    #[storage(read, write)]
    fn double_input(value: u64) -> u64 {
        let new_value = value * 2;
        storage.value.write(new_value);
        new_value
    }
}
// ANCHOR_END: target
