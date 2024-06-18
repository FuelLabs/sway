library;

mod req;

#[allow(dead_code)]
// ANCHOR: assert
fn subtract(a: u64, b: u64) -> u64 {
    assert(b <= a);
    a - b
}
// ANCHOR_END: assert

#[allow(dead_code)]
fn reverts() {
    // ANCHOR: revert
    revert(42);
    // ANCHOR_END: revert
}

#[allow(dead_code)]
// ANCHOR: assert_eq
fn compare_eq(a: u64, b: u64) {
    assert_eq(a, b);
    // code
}
// ANCHOR_END: assert_eq

#[allow(dead_code)]
// ANCHOR: assert_ne
fn compare_ne(a: u64, b: u64) {
    assert_ne(a, b);
    // code
}
// ANCHOR_END: assert_ne