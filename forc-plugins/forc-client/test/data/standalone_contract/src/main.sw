contract;

storage {
    value: u8 = 5,
}

abi MyContract {
    fn test_function() -> bool;

    #[storage(read)]
    fn test_function_read() -> u8;

    #[storage(read, write)]
    fn test_function_write(value: u8) -> u8;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }

    #[storage(read)]
    fn test_function_read() -> u8 {
        storage.value.read()
    }

    #[storage(read, write)]
    fn test_function_write(value: u8) -> u8 {
        storage.value.write(value);
        storage.value.read()
    }
}
