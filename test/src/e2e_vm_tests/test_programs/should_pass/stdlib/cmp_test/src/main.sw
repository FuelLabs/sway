script;

use std::u128::*;

fn test_cmp_u8() {

}

fn test_cmp_u16() {

}

fn test_cmp_u32() {

}

fn test_cmp_u64() {

}

fn test_cmp_u128() {
    let first = U128::from((0, 0));
    let second = U128::from((0, 1));
    let max = first.max(second);
    assert(max.upper() == 0);
    assert(max.lower() == 1);
}

fn test_cmp_u256() {

}

fn main() -> bool {
    test_cmp_u8();
    test_cmp_u16();
    test_cmp_u32();
    test_cmp_u64();
    test_cmp_u128();
    test_cmp_u256();

    true
}