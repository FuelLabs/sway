script;
use increment_abi::Incrementor;
use std::constants::ETH_ID;
fn main() {
    let abi = abi(Incrementor, 0x2b613882f0c2cfd1a18ceb7fd8b9579d7c475ad613991db46a3525932c2984e3);
    abi.initialize {
        gas: 10000, coins: 0, asset_id: ETH_ID
    }
    (0); // comment this line out to just increment without initializing
    abi.increment {
        gas: 10000, coins: 0, asset_id: ETH_ID
    }
    (5);
    let result = abi.increment {
        gas: 10000, coins: 0, asset_id: ETH_ID
    }
    (5);
    log(result);
}

fn log(input: u64) {
    asm(r1: input, r2: 42) {
        log r1 r2 r2 r2;
    }
}
