contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        f()
    }
}

#[cfg(experimental_aligned_and_dynamic_storage = false)]
#[storage(read)]
fn f() -> bool {
    let _ = __state_load_word(b256::zero());
    true
}

#[cfg(experimental_aligned_and_dynamic_storage = true)]
#[storage(read)]
fn f() -> bool {
    let _ = __state_load_word(b256::zero(), 0);
    true
}
