script;

use core::codec::*;
use core::codec::AbiEncode;
use std::vec::*;

struct SS<T> {
    ss: T
}

struct S {
    a: u64,
    b: u32,
    c: u16,
    d: u8,
    e: Vec<u64>,
    f: str,
    g: u256
}

fn main() -> u64 {
    let mut e = Vec::new();
    e.push(1);
    e.push(2);
    e.push(3);
    __log(S{
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e,
        f: "sway",
        g: u256::max()
    });

    __log(SS{
        ss: 1u64
    });
    
    1
}
