library single_lib_test;

use std::assert::assert;

#[test]
fn test_meaning_of_life() {
    let meaning = 6 * 7;
    assert(meaning == 42);
}
