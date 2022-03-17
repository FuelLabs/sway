script;

use std::chain::assert;

fn main() -> bool {
    let mut counter1 = 0u64;
    // test standard while loop
    while counter1 < 10 {
        counter1 = counter1 + 1;
    }
    assert(counter1 == 10);

    // test early exit from loop with manual "break" (by invalidating the condition)
    let mut counter2 = 0u64;
    let mut n = 0u64;
    while counter2 < 10 {
        if counter2 == 3 {
            counter2 = 11;
        } else {
            counter2 = counter2 + 1;
            n = n + 1;
        }
    }

    assert(counter2 == 11 && n == 3);

    true
}
