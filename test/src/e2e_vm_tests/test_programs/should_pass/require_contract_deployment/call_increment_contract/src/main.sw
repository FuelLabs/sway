script;

use increment_abi::Incrementor;
use std::chain::assert;

fn main() {
    let abi = abi(Incrementor, 0x4c30f62e9947cff714c802afc0c900de272dbeec57ae12ed96aacbfd32c3e3a8);
    abi.initialize {
        gas: 10000
    }
    (0); // comment this line out to just increment without initializing
    abi.increment {
        gas: 10000
    }
    (5);
    abi.increment {
        gas: 10000
    }
    (5);
    let result = abi.get {
        gas: 10000
    }
    ();
    assert(result == 10);
    log(result);
}

fn log(input: u64) {
    asm(r1: input, r2: 42) {
        log r1 r2 r2 r2;
    }
}
