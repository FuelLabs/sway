script;

// if test passes, return true

use std::hash::HashMethod;
use std::hash::hash_pair;
use std::types::B512;



fn main() -> bool {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x5555555555555555555555555555555555555555555555555555555555555555;

    let mut first_test: bool = false;
    let mut second_test: bool = false;
    let mut third_test: bool = true;

    // let mut a = ~B512::new();
    // if (a.hi == 0) && (a.lo == 0) {
    //     first_test = true;
    // };

    // ahi = hi_bits;
    // a.lo = lo_bits;
    // if (a.hi == hi_bits) && (a.lo == lo_bits) {
    //     second_test = true;
    // };

    let b = ~B512::from_b256(hi_bits, lo_bits);
    if (b.hi == hi_bits) && (b.lo == lo_bits) {
        second_test = true;
    };


    first_test && second_test
}