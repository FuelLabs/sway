contract;

abi A {
    const ID: u32 = 1;
    fn foo() -> u32;
}

impl A for Contract {
    const ID: u32 = 2;

    fn foo() -> u32 {
        Self::ID
    }
}

abi B {
    const ID: u32 = 3;
    fn foo() -> u32;
}

impl B for Contract {
    const ID: u32 = 4;

    fn foo() -> u32 {
        Self::ID
    }
}

// TODO: Enable the asserts or adapt the tests once https://github.com/FuelLabs/sway/issues/6306 is implemented.
#[test]
fn test() {
    let a = abi(A, CONTRACT_ID);
    // assert_eq(2, a.foo());

    let b = abi(B, CONTRACT_ID);
    // assert_eq(4, b.foo());
}
