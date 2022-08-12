script;

use std::assert::assert;

fn main() -> bool {
    let mut counter = 0;
    // test standard while loop:
    while counter < 10 {
        counter = counter + 1;
    }
    assert(counter == 10);

    // test early exit from loop with manual "break" (by invalidating the condition):
    let mut counter_2 = 0;
    let mut counter_3 = 0;
    while counter_2 < 10 {
        if counter_2 == 3 {
            // ensure that condition is now invalid:
            counter_2 = 10;
        } else {
            counter_2 = counter_2 + 1;
            counter_3 = counter_3 + 1;
        }
    }

    assert(counter_2 == 10 && counter_3 == 3);

    // test nested loops:
    let mut counter_4 = 0;
    let mut counter_5 = 0;

    while counter_4 < 7 {
        while counter_5 < 11 {
            counter_5 = counter_5 + 1;
        }
        counter_4 = counter_4 + 1;
    }
    assert(counter_5 == 11);
    assert(counter_4 == 7);

    // test while loop expression
    let result = while true { break; };

    true
}
