library uint_tests;

use ::my_math::*;

use std::assert::assert;

pub fn basic_unit_tests() {
    assert(100.my_add(99) == 199);
    assert(3.my_mul(4) == 12);
    assert(5.my_double() == 10);
    assert(5.my_pow_2() == 25);
}
