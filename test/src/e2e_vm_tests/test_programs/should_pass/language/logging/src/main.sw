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

struct CustomAbiEncode {
}

impl AbiEncode for CustomAbiEncode {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push(77);
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

    // These must compile when experimental-new-encoding is not set
    // and fail when it is set
    let not_encodable = NotAutoEncodable{
        p: asm(size: 1) {
            aloc size;
            hp: raw_ptr
        }
    };
    log(not_encodable);
    require(true, not_encodable);

    1
}
