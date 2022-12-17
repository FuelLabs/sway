predicate;

use std::assert::assert;

fn main() -> bool {
    let a = 42;
    let b = 11;
    let c = a +  b;
    assert(c == 53);
    true
}
