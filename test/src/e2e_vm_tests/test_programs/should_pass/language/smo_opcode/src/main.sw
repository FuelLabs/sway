script;

use std::constants::ZERO_B256;

fn main() -> bool {
    asm(recipient: ZERO_B256, rB: 0, output, coins: 0) {
        smo recipient rB coins output;
    }
    true
}
