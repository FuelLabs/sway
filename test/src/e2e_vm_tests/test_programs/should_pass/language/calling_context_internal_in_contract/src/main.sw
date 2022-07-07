contract;

#[context(internal_only)]
fn bar() {}

abi TestContract {
    fn foo();
}

impl TestContract for Contract {
    #[context(internal_only)]
    fn foo() {
      bar()
    }
}
