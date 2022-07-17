script;

use std::assert::assert;
use std::u256::U256;
use core::num::*;

fn main() -> U256 {
    let zero = ~U256::from(0, 0, 0, 0);
    let one = ~U256::from(0, 0, 0, 1);
    let two = ~U256::from(0, 0, 0, 2);
    let max_u64 = ~U256::from(0, 0, 0, ~u64::max());

    ~U256::from(~u64::max(), 0, 0, 0) * ~U256::from(~u64::max(), 0, 0, 0)
}
