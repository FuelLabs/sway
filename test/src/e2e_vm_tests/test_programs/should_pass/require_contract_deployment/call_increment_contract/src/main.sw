script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0xb735d4f30161b1898a99816eac7c4ed02b80253af1f4ce6d10b5773bd3004e9e);
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
