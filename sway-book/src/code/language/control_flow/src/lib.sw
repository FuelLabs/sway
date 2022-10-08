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

fn single_loop() {
    // ANCHOR: single_loop
    let mut counter = 0;
    while counter < 10 {
        counter += 1;
    }
    // ANCHOR_END: single_loop
}

fn nested_loop() {
    // ANCHOR: nested_loop
    while true {
        // computation here
        while true {
            // more computation here
        }
    }
    // ANCHOR_END: nested_loop
}

// ANCHOR: break_example
fn break_example() -> u64 {
    let mut counter = 1;
    let mut sum = 0;
    let num = 10;
    while true {
        if counter > num {
            break;
        }
        sum += counter;
        counter += 1;
    }
    sum // 1 + 2 + .. + 10 = 55
}
// ANCHOR_END: break_example

// ANCHOR: continue_example
fn continue_example() -> u64 {
    let mut counter = 0;
    let mut sum = 0;
    let num = 10;
    while counter < num {
        counter += 1;
        if counter % 2 == 0 {
            continue;
        }
        sum += counter;
    }
    sum // 1 + 3 + .. + 9 = 25
}
// ANCHOR_END: continue_example
