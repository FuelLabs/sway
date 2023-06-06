contract;

storage {
    var: u64 = 0,
}

// ANCHOR: read
#[storage(read)]
// ANCHOR_END: read
fn read() {
    let variable = storage.var.read();
}

// ANCHOR: write
#[storage(write)]
// ANCHOR_END: write
fn write() {
    storage.var.write(storage.var.read() + 1);
}

// ANCHOR: read_write
#[storage(read, write)]
// ANCHOR_END: read_write
fn read_write() {
    let var = storage.var.read();
    storage.var.write(var + 1);
}

fn example() {
    // ANCHOR: example
    let bar: str[4] = "sway";
    let baz: bool = true;
    // ANCHOR_END: example
}

abi MyContract {
    // ANCHOR: payable
    #[payable]
    fn deposit();
    // ANCHOR_END: payable
}

// ANCHOR: success_test
#[test]
fn equal() {
    assert_eq(1 + 1, 2);
}
// ANCHOR_END: success_test

// ANCHOR: revert_test
#[test(should_revert)]
fn unequal() {
    assert_eq(1 + 1, 3);
}
// ANCHOR_END: revert_test

// ANCHOR: revert_code_test
#[test(should_revert = "18446744073709486084")]
fn assert_revert_code() {
    assert(1 + 1 == 3);
}

#[test(should_revert = "42")]
fn custom_revert_code() {
    revert(42);
}
// ANCHOR_END: revert_code_test

// ANCHOR: never_inline
#[inline(never)]
fn foo() {}
// ANCHOR_END: never_inline

// ANCHOR: always_inline
#[inline(always)]
fn bar() {}
// ANCHOR_END: always_inline
