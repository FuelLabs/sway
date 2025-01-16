script;

use std::u128::*;

fn test_totalord_u8() {
    let a = 10u8;
    let b = 20u8;

    let max = a.max(b);
    assert_eq(max, 20u8);

    let min = a.min(b);
    assert_eq(min, 10u8);

    let c = 30u8;
    let d = 30u8;

    let max = c.max(d);
    assert_eq(max, 30u8);

    let min = c.min(d);
    assert_eq(min, 30u8);
}

fn test_totalord_u16() {
    let a = 10u16;
    let b = 20u16;

    let max = a.max(b);
    assert_eq(max, 20);

    let min = a.min(b);
    assert_eq(min, 10);

    let c = 30u16;
    let d = 30u16;

    let max = c.max(d);
    assert_eq(max, 30);

    let min = c.min(d);
    assert_eq(min, 30);
}

fn test_totalord_u32() {
    let a = 10u32;
    let b = 20u32;

    let max = a.max(b);
    assert_eq(max, 20u32);

    let min = a.min(b);
    assert_eq(min, 10u32);

    let c = 30u32;
    let d = 30u32;

    let max = c.max(d);
    assert_eq(max, 30u32);

    let min = c.min(d);
    assert_eq(min, 30u32);
}

fn test_totalord_u64() {
    let a = 10;
    let b = 20;

    let max = a.max(b);
    assert_eq(max, 20);

    let min = a.min(b);
    assert_eq(min, 10);

    let c = 30;
    let d = 30;

    let max = c.max(d);
    assert_eq(max, 30);

    let min = c.min(d);
    assert_eq(min, 30);
}

fn test_totalord_u128() {
    let a = U128::from((0, 0));
    let b = U128::from((0, 1));

    let max = a.max(b);
    assert(max.upper() == 0);
    assert(max.lower() == 1);

    let min = a.min(b);
    assert(min.upper() == 0);
    assert(min.lower() == 0);

    let c = U128::from((2, 2));
    let d = U128::from((2, 2));

    let max = c.max(d);
    assert(max.upper() == 2);
    assert(max.lower() == 2);

    let min = c.min(d);
    assert(min.upper() == 2);
    assert(min.lower() == 2);
}

fn test_totalord_u256() {
    let a = 0x01u256;
    let b = 0x02u256;

    let max = a.max(b);
    assert_eq(max, 0x02u256);

    let min = a.min(b);
    assert_eq(min, 0x01u256);

    let c = 0x03u256;
    let d = 0x03u256;

    let max = c.max(d);
    assert_eq(max, 0x03u256);

    let min = c.min(d);
    assert_eq(min, 0x03u256);
}

fn main() -> bool {
    test_totalord_u8();
    test_totalord_u16();
    test_totalord_u32();
    test_totalord_u64();
    test_totalord_u128();
    test_totalord_u256();

    true
}
