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

enum E {
    A: SS<u64>,
    B: ()
}

enum F {
    A: ()
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

    __log(E::A(SS{
        ss: 1u64
    }));
    __log(E::B);

    match E::B {
        E::A(x) => __log(x),
        E::B(x) => __log(x),
    }

    match F::A {
        F::A => {}
    }

    1
}
