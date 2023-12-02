script;

use core::codec::*;

struct S {
    a: u64
}

fn main() {
    let s = S{
        a: 1
    };
    let buffer = encode(s);
    __log(buffer);
}
