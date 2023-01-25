script;

use core::ops::*;
use lib_vec_test::test_all;

fn main() -> bool {
    test_all::<u8>(0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8);

    true
}
