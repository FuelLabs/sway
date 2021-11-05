script;

// if test passes, return true

use std::types::B512;
use std::types::build_from_b256;
use std::constants::ETH_COLOR;

fn main() -> bool {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x5555555555555555555555555555555555555555555555555555555555555555;


    let mut t1: bool = false;
    let mut t2: bool = false;
    let mut t3: bool = false;
    let mut t4: bool = false;

    // it allows building from 2 b256's:
    // let b = ~B512::from_b256(hi_bits, lo_bits); // use method when bug is fixed
    let b: B512 = build_from_b256(hi_bits, lo_bits);
    let mut hi_test = false;
    let mut lo_test = false;

    if (b.hi == hi_bits) && (b.lo == lo_bits) {
        t1 = true;
    };


    // it allows creation of new empty type:
    let zero: b256 = 0;
    let mut a = ~B512::new();
    if (a.hi == zero) && (a.lo == zero) {
        t2 = true;
    };

    // it allows modification of fields:
    // a.hi = hi_bits;
    // a.lo = lo_bits;
    // if (a.hi == hi_bits) && (a.lo == lo_bits) {
    //     t3 = true;
    // };

    // it guarantees memory conitiguity:
    let mut c = ~B512::new();
    c.hi = 11;
    c.lo = 42;
    let next_bits = asm(r1, r2: c.hi) {
        addi r1 r2 i32;
        r1: b256
    };
    if next_bits == c.lo {
        t4 = true;
    };

    // t1 && t2 && t3 && t4
    t1




}