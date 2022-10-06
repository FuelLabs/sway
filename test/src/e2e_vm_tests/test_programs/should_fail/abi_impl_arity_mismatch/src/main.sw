contract;

abi MyContract {
    fn f();
}

impl MyContract for Contract {
    fn f(nondeclared_param : u64) -> bool {
        true
    }
}
