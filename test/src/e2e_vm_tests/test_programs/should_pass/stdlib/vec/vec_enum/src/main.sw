script;

use core::ops::*;
use lib_vec_test::test_all;

enum SimpleEnum {
    X: (),
    Y: b256,
    Z: (b256, b256),
}

impl Eq for SimpleEnum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (SimpleEnum::X, SimpleEnum::X) => true,
            (SimpleEnum::Y(y0), SimpleEnum::Y(y1)) => y0 == y1,
            (SimpleEnum::Z(z0), SimpleEnum::Z(z1)) => z0.0 == z1.0 && z0.1 == z1.1,
            _ => false,
        }
    }
}

impl Ord for SimpleEnum {
    fn gt(self, other: Self) -> bool {
        // X > Y > Z
        match (self, other) {
            (_, SimpleEnum::X) => false,
            (SimpleEnum::Y(y0), SimpleEnum::Y(y1)) => y0 > y1,
            (SimpleEnum::Z(z0), SimpleEnum::Z(z1)) => z0.0 > z1.0,
            (SimpleEnum::Z(_), _) => false,
            _ => true,
        }
    }

    fn lt(self, other: Self) -> bool {
        // Z < Y < X
        match (self, other) {
            (SimpleEnum::X, _) => false,
            (SimpleEnum::Y(y0), SimpleEnum::Y(y1)) => y0 < y1,
            (SimpleEnum::Y(_), SimpleEnum::Z(_)) => false,
            (SimpleEnum::Z(z0), SimpleEnum::Z(z1)) => z0.0 < z1.0,
            _ => true,
        }
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
    test_all::<SimpleEnum>(
        SimpleEnum::Y(B256_0),
        SimpleEnum::X,
        SimpleEnum::Z((B256_1, B256_2)),
        SimpleEnum::Y(B256_1),
        SimpleEnum::Y(B256_2),
        SimpleEnum::Z((B256_3, B256_4)),
        SimpleEnum::Z((B256_5, B256_5)),
        SimpleEnum::Y(B256_8),
        SimpleEnum::X,
    );

    true
}
