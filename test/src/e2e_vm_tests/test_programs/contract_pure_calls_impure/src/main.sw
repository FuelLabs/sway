contract;

abi ImpurityTest {
    fn impure_func() -> bool;
}

impl ImpurityTest for Contract {
    fn impure_func() -> bool {
        foo();
        true
    }
}

impure fn foo() {}
