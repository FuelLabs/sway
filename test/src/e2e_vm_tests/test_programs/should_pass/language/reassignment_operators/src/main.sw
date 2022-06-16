script;

use std::assert::assert;

fn main() -> u64 {
    let mut a = 0;
    
    a += 99;
    assert(a == 99);

    a -= 5;
    assert(a == 94);

    a *= 2;
    assert(a == 188);

    a /= 47;
    assert(a == 4);

    a = 999;

    a >>= 1;
    assert(a == 499);

    a <<= 2;
    assert(a == 1996);

    1
}
