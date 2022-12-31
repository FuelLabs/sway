contract;

abi CalledContract {
    #[storage(write)]
    fn set_value(new_value: u64);
    #[storage(read)]
    fn get_value() -> u64;
    #[storage(write)]
    fn set_b256_value(new_value: b256);
    #[storage(read)]
    fn get_b256_value() -> b256;
}

storage {
    value: u64 = 0,
    value_b256: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000,
}

// Test contract for low level calls. A calling script calls "set_value" via a low-level call,
// and then checks the call was successful by calling "get_value".
impl CalledContract for Contract {
    #[storage(write)]
    fn set_value(new_value: u64) {
        storage.value = new_value;
    }

    #[storage(read)]
    fn get_value() -> u64 {
        storage.value
    }

    #[storage(write)]
    fn set_b256_value(new_value: b256) {
        storage.value_b256 = new_value;
    }

    #[storage(read)]
    fn get_b256_value() -> b256 {
        storage.value_b256
    }
}
