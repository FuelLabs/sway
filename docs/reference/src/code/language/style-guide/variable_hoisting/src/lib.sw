library;

#[allow(dead_code)]
// ANCHOR: hoisting_variables
fn hoisting() -> u64 {
    let a = 1;
    let b = 2;

    // code

    a + b
}
// ANCHOR_END: hoisting_variables

#[allow(dead_code)]
// ANCHOR: grouping_variables
fn grouping() -> u64 {
    let a = 1;
    // code that uses `a`

    let b = 2;

    // remaining code

    a + b
}
// ANCHOR_END: grouping_variables
