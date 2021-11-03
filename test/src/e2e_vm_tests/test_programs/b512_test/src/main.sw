script;

// if test passes, return true

use std::types::B512;
use std::types::build_from_b256s;

fn main() -> bool {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x5555555555555555555555555555555555555555555555555555555555555555;

    let mut first_test: bool = false;
    // let mut second_test: bool = false;
    // let mut third_test: bool = false;

    // let b = ~B512::from_b256(hi_bits, lo_bits); // use method when bug is fixed
    let b: B512 = build_from_b256s(hi_bits, lo_bits);
    let mut hi_test = false;
    let mut lo_test = false;

    if (b.lo == hi_bits) && (b.hi == lo_bits) {
        first_test = true;
    };

    first_test


    // let mut a = ~B512::new();
    // if (a.hi == 0) && (a.lo == 0) {
    //     first_test = true;
    // };

    // ahi = hi_bits;
    // a.lo = lo_bits;
    // if (a.hi == hi_bits) && (a.lo == lo_bits) {
    //     second_test = true;
    // };




    // add test to prove memory conitiguity



}