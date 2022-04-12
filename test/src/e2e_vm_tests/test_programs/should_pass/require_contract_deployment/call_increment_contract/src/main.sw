script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let abi = abi(Incrementor, 0x386b732f205fd34c97c5914ddd0f7356c5b923229b1cb39e84acd762d62e69c6);
    abi.initialize(0); // comment this line out to just increment without initializing
    abi.increment(5);
    abi.increment(5);
    let result = abi.get();
    assert(result == 10);
    log(result);

    true
}

fn log(input: u64) {
    asm(r1: input, r2: 42) {
        log r1 r2 r2 r2;
    }
}
