script;

use std::chain::assert;

fn f(cond: bool) -> u64 {
    if cond {
        10
    } else {
        20
    }
}

fn main() {
    f(true);
    assert(f(false) == 20);
}
