script;

use increment_abi::Incrementor;
use dynamic_contract_call::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x080ca4b6a4661d3cc2138f733cbe54095ce8b910eee73d913c1f43ecad6bf0d2;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x227e9f3c58984166b7036d7d661cad9630ef49d169965e0e7d2f4893c736c6d8;

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
