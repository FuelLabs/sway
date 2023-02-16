script;

dep primitive_tests;
dep adt_tests;
dep complex_tests;

use primitive_tests::*;
use adt_tests::*;
use complex_tests::*;

fn main() -> u64 {
    simple_numbers_test();
    simple_tuples_test();
    point_test();
    crazy_point_test();
    variable_not_found_test();
    nested_match_tests();
    enum_match_exp_bugfix_test();
    enum_match_exp_bugfix_test2();

    42u64
}
