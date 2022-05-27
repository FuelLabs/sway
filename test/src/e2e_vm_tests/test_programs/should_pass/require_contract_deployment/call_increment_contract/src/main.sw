script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0x8a0c7a6ac575c531bdd6354517b3f7b5b0a58196daec3b4e316955276dfb5066);
    the_abi.initialize(0); // comment this line out to just increment without initializing
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
