script;

mod primitive_tests;
mod adt_tests;
mod complex_tests;
mod or_patterns;

use primitive_tests::*;
use adt_tests::*;
use complex_tests::*;
use or_patterns::*;

fn main() -> u64 {
    simple_numbers_test();
    simple_tuples_test();
    point_test();
    crazy_point_test();
    variable_not_found_test();
    nested_match_tests();
    enum_match_exp_bugfix_test();
    enum_match_exp_bugfix_test2();
    or_patterns_test();

    42u64
}
