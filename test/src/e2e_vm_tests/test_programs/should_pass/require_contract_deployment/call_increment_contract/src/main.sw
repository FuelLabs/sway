script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(Incrementor,0x2fa39fe61cb782b27091ad143f1d7a6e5eddd22d044cf1ab754dcc1218e182a9 );
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
