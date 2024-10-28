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

// TODO: Enable the asserts or adapt the tests once https://github.com/FuelLabs/sway/issues/6306 is implemented.
#[test]
fn test() {
    let a = abi(A, CONTRACT_ID);
    // assert_eq(3, a.foo());

    let b = abi(B, CONTRACT_ID);
    // assert_eq(5, b.foo());
}
