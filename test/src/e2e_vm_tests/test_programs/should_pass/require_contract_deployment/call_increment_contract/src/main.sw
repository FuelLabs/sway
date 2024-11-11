script;

use increment_abi::Incrementor;
use dynamic_contract_call::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xd1b4047af7ef111c023ab71069e01dc2abfde487c0a0ce1268e4f447e6c6e4c2;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xc906accda9e1e516ee3122770217d913edca1ae6e3b8bad15c2c1a53ebf5e13e; // AUTO-CONTRACT-ID ../../test_contracts/increment_contract --release

fn main() -> bool {
    let the_abi = abi(Incrementor, CONTRACT_ID);

    let initial = the_abi.get();

    let result = the_abi.increment(5);
    assert(result == initial + 5);

    let result = the_abi.increment_or_not(None);
    assert(result == initial + 5);

    let result = the_abi.increment(5);
    assert(result == initial + 10);

    let result = the_abi.get();
    assert(result == initial + 10);

    log(result);

    // Call the fallback fn
    let result = dynamic_contract_call(CONTRACT_ID);
    assert(result == 444444444);

    true
}

fn log(input: u64) {
    asm(r1: input, r2: 42) {
        log r1 r2 r2 r2;
    }
}
