script;

use std::{address::Address, identity::Identity, assert::assert, revert::revert};

const B1 = Address {
    value: 0x0100000000000000000000000000000000000000000000000000000000000010
};

pub fn addr_of<T>(val: T) -> u64 {
    if !__is_reference_type::<T>() {
        revert(0);
    }
    asm(ptr: val) {
        ptr: u64
    }
}

fn main() {
    let sender = Identity::Address(B1);
    assert (__addr_of(sender) == addr_of(sender));
}
