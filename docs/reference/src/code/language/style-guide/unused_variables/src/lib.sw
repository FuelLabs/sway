library;

#[allow(dead_code)]
fn call() -> (u64, u64) {
    // Random data for demonstration in the subsequent functions
    (1, 2)
}

#[allow(dead_code)]
// ANCHOR: named_unused_variable
fn named_unused_variable() -> u64 {
    let (timestamp, deposit_amount) = call();

    deposit_amount
}
// ANCHOR_END: named_unused_variable

#[allow(dead_code)]
// ANCHOR: unnamed_unused_variable
fn unnamed_unused_variable() -> u64 {
    let (_, deposit_amount) = call();

    deposit_amount
}
// ANCHOR_END: unnamed_unused_variable
