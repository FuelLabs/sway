contract;

struct B {
    b: str[1],
}

storage {
    not_ok_1: str[1] = __to_str_array("a"),
    not_ok_2: B = B { b: __to_str_array("b") },

    ok_1: str[8] = __to_str_array("abcdefgh"),
    ok_2: u64 = 0,
}

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}
