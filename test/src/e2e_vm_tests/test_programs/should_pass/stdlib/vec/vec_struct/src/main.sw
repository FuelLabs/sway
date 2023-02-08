script;

use core::ops::*;
use lib_vec_test::test_all;

struct SimpleStruct {
    x: u32,
    y: b256,
}

impl Eq for SimpleStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Ord for SimpleStruct {
    fn gt(self, other: Self) -> bool {
        self.x > other.x
    }

    fn lt(self, other: Self) -> bool {
        self.x < other.x
    }
}

const B256_0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const B256_1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const B256_2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
const B256_3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
const B256_4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
const B256_5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
const B256_6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
const B256_7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
const B256_8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

fn main() -> bool {
    test_all::<SimpleStruct>(
        SimpleStruct { x: 0_u32, y: B256_0 },
        SimpleStruct { x: 1_u32, y: B256_1 },
        SimpleStruct { x: 2_u32, y: B256_2 },
        SimpleStruct { x: 3_u32, y: B256_3 },
        SimpleStruct { x: 4_u32, y: B256_4 },
        SimpleStruct { x: 5_u32, y: B256_5 },
        SimpleStruct { x: 6_u32, y: B256_6 },
        SimpleStruct { x: 7_u32, y: B256_7 },
        SimpleStruct { x: 8_u32, y: B256_8 },
    );

    true
}
