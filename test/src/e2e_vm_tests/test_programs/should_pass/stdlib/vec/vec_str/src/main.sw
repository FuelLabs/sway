script;

use core::ops::*;
use std::hash::sha256;
use lib_vec_test::test_all;

impl Eq for str[4] {
    fn eq(self, other: Self) -> bool {
        sha256(self) == sha256(other)
    }
}

fn main() -> bool {
    test_all::<str[4]>(
        "fuel",
        "john",
        "nick",
        "adam",
        "emma",
        "sway",
        "gmgn",
        "kekw",
        "meow",
    );

    true
}

