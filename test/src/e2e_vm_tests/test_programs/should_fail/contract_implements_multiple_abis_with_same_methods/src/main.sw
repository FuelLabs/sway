contract;

abi A {
    fn foo() -> u64;
    fn foo2();
}

abi B {
    fn foo() -> u64;
    fn foo2();
}

impl A for Contract {
    fn foo() -> u64 {
        3
    }

    fn foo2() {}
}

impl B for Contract {
    fn foo() -> u64 {
        2
    }

    fn foo2() {}
}
