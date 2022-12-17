contract;

abi CalledContract {
    #[storage(write)]
    fn set_value(new_value: u64);
    #[storage(read)]
    fn get_value() -> u64;
}

storage {
    value: u64 = 0,
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
}
