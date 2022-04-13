script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let abi = abi(Incrementor, 0xc07e767d214aa07d772d589fa0da89a38b93da6fd8f329877c7423cbdefd815d);
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
