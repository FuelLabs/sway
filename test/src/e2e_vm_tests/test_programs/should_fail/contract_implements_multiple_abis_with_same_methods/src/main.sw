contract;

abi A {
    fn foo() -> u64;
}

abi B {
    fn foo() -> u64;
}

impl A for Contract {
    fn foo() -> u64 {
        3
    }
}

impl B for Contract {
    fn foo() -> u64 {
        5
    }
}
