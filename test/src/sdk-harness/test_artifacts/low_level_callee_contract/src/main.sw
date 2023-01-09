contract;

abi CalledContract {
    #[storage(write)]
    fn set_value(new_value: u64);
    #[storage(write)]
    fn set_value_multiple(a: u64, b: u64);
    #[storage(write)]
    fn set_value_multiple_complex(a: MyStruct, b: str[4]);
    #[storage(read)]
    fn get_value() -> u64;
    #[storage(write)]
    fn set_b256_value(new_value: b256);
    #[storage(read)]
    fn get_b256_value() -> b256;
    #[storage(read)]
    fn get_str_value() -> str[4];
    #[storage(read)]
    fn get_bool_value() -> bool;
}

pub struct MyStruct {
    a: bool,
    b: [u64; 3],
}

storage {
    value: u64 = 0,
    value_b256: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000,
    value_str: str[4] = "none",
    value_bool: bool = false,
}

// Test contract for low level calls. A calling script calls "set_value" via a low-level call,
// and then checks the call was successful by calling "get_value".
impl CalledContract for Contract {
    #[storage(write)]
    fn set_value(new_value: u64) {
        storage.value = new_value;
    }

    #[storage(write)]
    fn set_value_multiple(a: u64, b: u64) {
        storage.value = a + b;
    }

    #[storage(write)]
    fn set_value_multiple_complex(a: MyStruct, b: str[4]) {
        //revert(999);
        storage.value = a.b[1];
        storage.value_str = b;
        storage.value_bool = a.a;
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

    #[storage(read)]
    fn get_str_value() -> str[4] {
        storage.value_str
    }

    #[storage(read)]
    fn get_bool_value() -> bool {
        storage.value_bool
    }
}
