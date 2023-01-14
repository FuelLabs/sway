contract;

abi MethodTest {
    fn wrong_return_type() -> bool;
    fn wrong_arg_type(x: bool) -> bool;
    fn wrong_arg_type_and_return_type(x: bool) -> bool;
}

impl MethodTest for Contract {
    fn wrong_return_type() -> u64 {
        return 1;
    }

    fn wrong_arg_type(x: u64) -> bool {
        return true;
    }

    fn wrong_arg_type_and_return_type(x: u64) -> u64 {
        return 1;
    }
}
