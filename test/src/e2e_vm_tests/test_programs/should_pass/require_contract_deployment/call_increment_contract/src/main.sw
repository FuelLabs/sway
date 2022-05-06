script;

use increment_abi::Incrementor;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0xeff8d28ce02d20aac8e32c811f1760f5031670d6a141bd7c0ee6ba594ac31355);
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
