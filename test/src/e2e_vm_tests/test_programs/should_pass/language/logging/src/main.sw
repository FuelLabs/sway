script;

use core::codec::*;

struct S {
}

fn main() {
    let s = S{};
    encode(s);
}

// check: script