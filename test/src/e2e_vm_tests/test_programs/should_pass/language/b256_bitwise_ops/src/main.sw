script;

use core::{num::*, ops::*};
use std::assert::assert;
use std::b256_ops::*;
use std::logging::log;

fn main() -> bool {
    let a: b256 = 0b1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001_1000000000000001;

    let b: b256 = 0b0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001_0000000100000001;

    let c: b256 = 0b0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001;

    let d: b256 = 0b1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000_1000000100000000;

    let e: b256 = 0b1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001_1000000100000001;

    let f: b256 = 0b1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000;

    let g: b256 = 0b0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000_0000000100000000;

    //0001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001
    let b256_1: b256 = 0x1111111111111111111111111111111111111111111111111111111111111111;

    // 0010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010
    let b256_2: b256 = 0x2222222222222222222222222222222222222222222222222222222222222222;

    //0011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011001100110011
    let b256_3: b256 = 0x3333333333333333333333333333333333333333333333333333333333333333;

    // 0100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100010001000100
    let b256_4: b256 = 0x4444444444444444444444444444444444444444444444444444444444444444;

    // 0101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101
    let b256_5: b256 = 0x5555555555555555555555555555555555555555555555555555555555555555;
    let b256_F: b256 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;

    ///////////////////////////////////////////////////////
    // test &, |, ^
    ///////////////////////////////////////////////////////

    assert(a & b == c);
    assert(a & c == c);
    assert(b & c == c);
    assert(a & d == f);
    assert(f & e == f);
    assert(b & d == g);
    assert(b256_F & b256_3 == b256_3);
    assert(b256_1 & b256_2 == ~b256::min());
    assert(b256_F & b256_2 == b256_2);

    assert(a | g == e);
    assert(a | d == e);
    assert(a | c == a);
    assert(c | f == a);
    assert(c | d == e);
    assert(b256_1 | b256_2 == b256_3);
    assert(b256_1 | b256_4 == b256_5);
    assert(b256_2 | b256_3 == b256_3);

    assert(a ^ b == d);
    assert(a ^ g == e);
    assert(b ^ d == a);
    assert(f ^ g == d);
    assert(b256_1 ^ b256_2 == b256_3);
    assert(b256_2 ^ b256_3 == b256_1);
    assert(b256_1 ^ b256_3 == b256_2);

    let zero = ~b256::min();
    let one: b256 = 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000001;

    let two: b256 = 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000010;

    let g: b256 = 0b0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000001_0000000000000000;

    let h: b256 = 0b0000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000_1000000000000000;

    let saturated: b256 = ~b256::max();

    let highest_bit_only: b256 = 0b1000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000;

    let all_but_highest_bit: b256 = 0b0111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111_1111111111111111;

    assert(one >> 1 == zero);
    assert(one << 1 == two);
    assert(two << 1 == 0x0000000000000000000000000000000000000000000000000000000000000004);
    assert(f << 1 == g);
    assert(c >> 1 == h);
    assert(0x0000000000000000000000000000000000000000000000000000000000000001 << 10 == 0x0000000000000000000000000000000000000000000000000000000000000400);
    assert(0x000000000000000000000000000000000000000000000000000000000AB1142C >> 3 == 0x0000000000000000000000000000000000000000000000000000000001562285);
    assert(saturated << 255 == highest_bit_only);
    assert(saturated >> 1 == all_but_highest_bit);
    assert(saturated >> 255 == one);
    assert(highest_bit_only >> 255 == one);
    assert(highest_bit_only >> 254 == two);

    ///////////////////////////////////////////////////////
    // test Ord
    ///////////////////////////////////////////////////////

    assert(one > zero);
    assert(two > one);
    assert(one < two);
    assert(two < saturated);
    assert(highest_bit_only < saturated);

    assert(saturated > highest_bit_only);
    assert(highest_bit_only > all_but_highest_bit);
    assert(saturated > 0x5555555555555555555555555555555555555555555555555555555555555555);
    assert(0x5555555555555555555555555555555555555555555555555555555555555555 > 0x4444444444444444444444444444444444444444444444444444444444444444);

    // test differences in only one word
    let foo: b256 = 0x0111111111111111_1111111111111111_1111111111111111_1111111111111111;
    let bar: b256 = 0x1111111111111111_0111111111111111_1111111111111111_1111111111111111;
    let baz: b256 = 0x1111111111111111_1111111111111111_0111111111111111_1111111111111111;
    let fiz: b256 = 0x1111111111111111_1111111111111111_1111111111111111_0111111111111111;

    let w: b256 = 0xF000000000000000_E000000000000000_E000000000000000_E000000000000000;
    let x: b256 = 0xE000000000000000_F000000000000000_E000000000000000_E000000000000000;
    let y: b256 = 0xE000000000000000_E000000000000000_F000000000000000_E000000000000000;
    let z: b256 = 0xE000000000000000_E000000000000000_E000000000000000_F000000000000000;

    assert(foo < bar);
    assert(bar < baz);
    assert(baz < fiz);
    assert(fiz > baz);
    assert(baz > bar);
    assert(bar > foo);

    assert(w > x);
    assert(x > y);
    assert(y > z);
    assert(z < y);
    assert(y < x);
    assert(x < w);

    // OrdEq
    assert(x <= w);
    assert(w >= x);
    assert(foo >= foo);
    assert(foo <= foo);
    assert(fiz >= baz);
    assert(foo <= fiz);

    ///////////////////////////////////////////////////////
    // test Add
    ///////////////////////////////////////////////////////

    let one_thousand = 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000001111101000;

    let two_thousand = 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000011111010000;

    let one_million = 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000001111_0100001001000000;

    let one_quintillion = 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000110111100000_1011011010110011_1010011101100100_0000000000000000;

    assert(one + one == two);

    assert(two + one == 0x0000000000000000_0000000000000000_0000000000000000_0000000000000003);

    assert(one_thousand + one == 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000001111101001);

    assert(one_million + one == 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000001111_0100001001000001);

    assert(one_quintillion + one == 0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000110111100000_1011011010110011_1010011101100100_0000000000000001);

    assert(all_but_highest_bit + one == highest_bit_only);

    assert(one + one + one == 0x0000000000000000_0000000000000000_0000000000000000_0000000000000003);

    assert(0xEFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF + one == 0b1111000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000);

    true
}
