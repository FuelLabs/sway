script;

use std::constants::ZERO_B256;

// a b256 is bigger than a word, so RETD should be used instead of RET.
fn main() -> b256 {
    let a = ZERO_B256;
    asm(r1: a, r2: ZERO_B256) {
        log r1 r2 zero zero;
        zero
    };
    return a;
}
