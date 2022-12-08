library assertions;

dep req;

// ANCHOR: assert
fn subtract(a: u64, b: u64) -> u64 {
    assert(b <= a);
    a - b
}
// ANCHOR_END: assert
fn reverts() {
    // ANCHOR: revert
    revert(42);
    // ANCHOR_END: revert
}
