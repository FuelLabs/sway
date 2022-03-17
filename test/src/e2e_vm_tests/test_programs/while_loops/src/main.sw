script;

use std::chain::assert;

fn main() -> bool {
    let mut counter1 = 0;
    // test standard while loop
    while counter1 < 10 {
        counter1 = counter1 + 1;
    }
    assert(counter1 == 10);

    // test early exit from loop with manual "break" (by invalidating the condition)
    let mut counter2 = 0;
    let mut counter3 = 0;
    while counter2 < 10 {
        if counter2 == 3 {
            // ensure that condition is now invalid:
            counter2 = 10;
        } else {
            counter2 = counter2 + 1;
            counter3 = counter3 + 1;
        }
    }

    assert(counter2 == 11 && counter3 == 3);

    true
}
