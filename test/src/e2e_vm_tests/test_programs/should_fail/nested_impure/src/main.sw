contract;

abi Abi {
    fn test();
}

impl Abi for Contract {
    fn test() {
        foo();
    }
}

fn foo() {
    bar();
    let _ = baz();
}

// Although annotated, with no args is pure.
#[storage()]
fn bar() {
    let _ = baz();
}

// Explicitly impure.
#[storage(read)]
fn baz() -> u64 {
    let _ = __state_load_word(b256::zero());
    5
}
