script;

use std::assert::assert;

fn main() {
    // This just demonstrates that the `xos` opcode has been added to the compiler.
    // Actually using the opcode to get a value is out of scope for the PR in which this was introduced, but will be added by a subsequent PR.
    // This test is NOT currently run in mod.rs because it would fail with:
    // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: OutputNotFound', test/src/e2e_vm_tests/harness.rs:84:10
    asm(slot: 0, type) {
        xos type slot;
        type: u64
    };
}
