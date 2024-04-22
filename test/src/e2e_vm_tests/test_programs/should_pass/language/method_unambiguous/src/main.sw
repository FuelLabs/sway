script;

use std::convert::From;

impl From<u8> for u256 {
    fn from(num: u8) -> Self {
        num.as_u256()
    }
}

impl From<u16> for u256 {
    fn from(num: u16) -> Self {
        num.as_u256()
    }
}

fn main() -> u64 {
    use std::assert::assert;

    let u256_value = u256::from(255_u8);
    assert(u256_value == 0x00000000000000000000000000000000000000000000000000000000000000ff_u256);

    let u256_value = u256::from(65535_u16);
    assert(u256_value == 0x000000000000000000000000000000000000000000000000000000000000ffff_u256);

    1
}
