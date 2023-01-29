script;

use core::*;

fn foo(init: u64, n: u64) -> u64 {
    let mut index = 0;
    let mut sum = 0;
    while index < n {
        if index % 2 == 0 {
            sum = sum + index;
        };
        index = index + 1;
    }
    sum + init 
}

fn main() -> bool {
    let x = foo(11, 4);
    assert(x == 13);
    true
}
