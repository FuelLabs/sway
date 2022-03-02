script;
use increment_abi::Incrementor;
use std::constants::ETH_ID;
fn main() {
    let abi = abi(Incrementor, 0x5b864e5e90c8c0acb8adb66197e6738a72c590742971f23584d0e35010f50dbd);
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
