script;

dep my_add;
dep my_mul;
dep my_math;
dep my_point;
dep uint_tests;
dep point_tests;

use std::{
    assert::assert,
    logging::log,
};

use my_add::*;
use my_mul::*;
use my_math::*;
use my_point::*;
use uint_tests::*;
use point_tests::*;

fn main() -> u64 {
    basic_unit_tests();
    basic_point_tests();

    42
}
