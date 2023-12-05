script;

use core::codec::*;

struct S {
    a: u64
}

fn main() -> u64 {
    let s = S{
        a: 1
    };
    let buffer = encode(s);
    let slice: raw_slice = buffer.as_raw_slice(); 
    __log(slice);
    0
}
