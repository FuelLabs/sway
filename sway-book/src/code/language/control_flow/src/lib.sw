library control_flow;

fn conditional() {
    // ANCHOR: conditional
    let number = 5;

    if number % 3 == 0 {
        // call function 1
    } else if number % 4 == 0 {
        // call function 2
    } else {
        // call function 3
    }
    // ANCHOR_END: conditional
}

// ANCHOR: compute
fn compute(deposit: u64) {
    let minimum_deposit_met = if 10 < deposit { true } else { false };
    // code
}
// ANCHOR_END: compute
