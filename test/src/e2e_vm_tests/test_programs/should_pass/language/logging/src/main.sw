script;

use std::codec::*;
use std::codec::AbiEncode;
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

struct CustomAbiEncode {
}

impl AbiEncode for CustomAbiEncode {
    fn is_memcopy() -> bool { false }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        77u64.abi_encode(buffer)
    }
}

struct NotAutoEncodable {
    p: raw_ptr
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
    __log(CustomAbiEncode {});

    1
}
