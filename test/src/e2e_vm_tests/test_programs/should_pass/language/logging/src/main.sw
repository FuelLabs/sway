script;

use core::codec::*;
use std::vec::*;

struct S {
    a: u64,
    b: u32,
    c: u16,
    d: u8,
    e: Vec<u64>
}

fn main() -> u64 {
    let mut e = Vec::new();
    e.push(1);
    e.push(2);
    e.push(3);
    let s = S{
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e
    };
    let buffer = encode(s);
    let slice: raw_slice = buffer.as_raw_slice(); 
    __log(slice);
    
    1
}
