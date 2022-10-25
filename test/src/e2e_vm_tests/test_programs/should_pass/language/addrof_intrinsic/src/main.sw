script;

use std::{address::Address, identity::Identity, assert::assert, revert::revert};

const B1 = Address {
    value: 0x0100000000000000000000000000000000000000000000000000000000000010
};

pub fn addr_of<T>(val: T) -> raw_ptr {
    if !__is_reference_type::<T>() {
        revert(0);
    }
    asm(ptr: val) {
        ptr: raw_ptr
    }
}

enum X {
     A: u32,
     B: u64,
}

fn main() {
    let sender = Identity::Address(B1);
    assert (__addr_of(sender) == addr_of(sender));

    let x = X::A(2);
    let y = X::B(22);
    assert(__addr_of(x) == addr_of(x));
    assert(__addr_of(x) != addr_of(y));

    let a = [1,2,3];
    assert(__addr_of(a) == addr_of(a));

    let b = "hello";
    assert(__addr_of(b) == addr_of(b));

    let c = (1, 2);
    assert(__addr_of(c) == addr_of(c));
}
