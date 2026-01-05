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
    fn is_encode_trivial() -> bool { false }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        77u64.abi_encode(buffer)
    }
}

struct NotAutoEncodable {
    p: raw_ptr
}

#[inline(never)]
fn local_log<T>(item: T) where T: AbiEncode {
    __log(item);
}

fn main() -> u64 {
    local_log(0u64);

    let mut e = Vec::new();
    e.push(1);
    e.push(2);
    e.push(3);

    local_log(S{
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e,
        f: "sway",
        g: u256::max()
    });
    local_log(SS{
        ss: 1u64
    });
    local_log(E::A(SS{
        ss: 1u64
    }));
    local_log(E::B);
    local_log(CustomAbiEncode {});

    1
}

#[test]
fn call_main() {
    main();
}