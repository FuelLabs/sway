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

#[storage(write)]
fn foo() {
    let _ = __state_store_word(b256::zero(), 0);
}
