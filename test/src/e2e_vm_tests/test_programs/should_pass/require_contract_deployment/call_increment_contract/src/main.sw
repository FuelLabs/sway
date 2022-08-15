script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0x0bb4d41501aed8a57965e65946157c401a1acfe30df3326a660d7b3288457ddd);
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
