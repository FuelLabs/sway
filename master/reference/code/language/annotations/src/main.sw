contract;

// ANCHOR: storage_namespace
storage {
    my_storage_namespace {
        // ANCHOR_END: storage_namespace
        var: u64 = 0,
    }
}

// ANCHOR: read
#[storage(read)]
// ANCHOR_END: read
fn read() {
    // ANCHOR: storage_namespace_access
    let variable = storage::my_storage_namespace.var.read();
    // ANCHOR_END: storage_namespace_access

}

// ANCHOR: write
#[storage(write)]
// ANCHOR_END: write
fn write() {
    storage::my_storage_namespace.var.write(storage::my_storage_namespace.var.read() + 1);
}

// ANCHOR: read_write
#[storage(read, write)]
// ANCHOR_END: read_write
fn read_write() {
    let var = storage::my_storage_namespace.var.read();
    storage::my_storage_namespace.var.write(var + 1);
}

fn example() {
    // ANCHOR: example
    let bar: str = "sway";
    let baz: bool = true;
    // ANCHOR_END: example
}

abi MyContract {
    // ANCHOR: payable
    #[payable]
    fn deposit();
    // ANCHOR_END: payable
}

// ANCHOR: allow_deadcode_annotation
#[allow(dead_code)]
fn unused_function() {}
// ANCHOR_END: allow_deadcode_annotation

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


// ANCHOR: allow_deprecated_annotation
#[deprecated(note = "This is deprecated.")]
struct DeprecatedStruct {}

#[allow(deprecated)]
fn using_deprecated_struct() {
    let _ = DeprecatedStruct {};
}
// ANCHOR_END: allow_deprecated_annotation
