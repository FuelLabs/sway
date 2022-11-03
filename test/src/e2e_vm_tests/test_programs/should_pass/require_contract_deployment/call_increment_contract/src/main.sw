script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0xd2e606693b6ee837ae7b719e54a64a5bef6366dcb9a46cbce5c8008dbb88ce54);
    the_abi.increment(5);
    the_abi.increment(5);
    let result = the_abi.get();
    assert(result == 10);
    log(result);

    true
}

fn log(input: u64) {
    asm(r1: input, r2: 42) {
        log r1 r2 r2 r2;
    }
}
