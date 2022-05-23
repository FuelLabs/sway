script;

use std::assert::assert;

fn main() -> bool {
    let mut counter1 = 0;
    while counter1 < 10 {
        if counter1 == 5 {
            break;
        }
        counter = counter + 1;
    }
    assert(counter1 == 5);

    let mut counter2 = 0;
    let mut counter3 = 0;
    while counter2 < 10 {
        counter2 = counter2 + 1;
        match counter2 {
            1 => {
                continue;
            },
            3 => {
                continue;
            },
            5 => {
                continue;
            },
            7 => {
                continue;
            },
            9 => {
                continue;
            }
        }
        counter3 = counter3 + 1;
    }
    assert(counter3 == 4);


    let mut counter2 = 0;
    let mut counter3 = 0;
    while counter2 < 10 {
        if counter3 > 2 {
            break;
        }
        counter2 = counter2 + 1;
        match counter2 {
            1 => {
                continue;
            },
            3 => {
                continue;
            },
            5 => {
                continue;
            },
            7 => {
                continue;
            },
            9 => {
                continue;
            }
        }
        counter3 = counter3 + 1;
    }
    assert(counter3 == 3);

    true
}
