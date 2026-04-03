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

fn bar() {
    let _ = baz();
}

// Explicitly impure.
#[cfg(experimental_aligned_and_dynamic_storage = false)]
#[storage(read)]
fn baz() -> u64 {
    let _ = __state_load_word(b256::zero());
    5
}

// Explicitly impure.
#[cfg(experimental_aligned_and_dynamic_storage = true)]
#[storage(read)]
fn baz() -> u64 {
    let _ = __state_load_word(b256::zero(), 0);
    5
}
