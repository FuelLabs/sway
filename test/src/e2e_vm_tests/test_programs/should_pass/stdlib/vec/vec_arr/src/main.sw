script;

use core::ops::*;
use lib_vec_test::test_all;

impl Eq for [u64; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}

fn main() -> bool {
    test_all::<[u64; 3]>(
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
        [9, 10, 11],
        [12, 13, 14],
        [15, 16, 17],
        [18, 19, 20],
        [21, 22, 23],
        [24, 25, 26],
    );

    true
}
