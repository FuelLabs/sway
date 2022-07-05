script;

use std::constants::ZERO_B256;

fn main() -> bool {
    asm(recipient: ZERO_B256, msg_len: 0, output: 0, coins: 0) {
        smo recipient msg_len coins output;
    }
    true
}
