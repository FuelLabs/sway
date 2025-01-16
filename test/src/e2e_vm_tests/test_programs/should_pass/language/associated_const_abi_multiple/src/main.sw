contract;

abi A {
    const ID: u32 = 1;

    fn a_foo() -> u32;
}

impl A for Contract {
    const ID: u32 = 2;

    fn a_foo() -> u32 {
        Self::ID
    }
}

abi B {
    const ID: u32 = 3;

    fn b_foo() -> u32;
}

impl B for Contract {
    const ID: u32 = 4;

    fn b_foo() -> u32 {
        Self::ID
    }
}

#[test]
fn test() {
    let a = abi(A, CONTRACT_ID);
    assert_eq(2, a.a_foo());

    let b = abi(B, CONTRACT_ID);
    assert_eq(4, b.b_foo());
}
