script;

use core::codec::*;

struct S {
    a: u64,
    b: u32,
    c: u16,
    d: u8
}

fn main() -> u64 {
    let s = S{
        a: 1,
        b: 2,
        c: 3,
        d: 4
    };
    let buffer = encode(s);
    let slice: raw_slice = buffer.as_raw_slice(); 
    __log(slice);
    
    1
}
