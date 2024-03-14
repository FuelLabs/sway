script;

use increment_abi::Incrementor;
use dynamic_contract_call::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x8255f28ff18b281b60cc4d50211adcec740c7060cacc38571468bf18556713c4;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const CONTRACT_ID = 0x8b2b2ebc6d2caf0a585195b3c0db99e4f03fa76c4ef5d727e387dad1724e6b67;
=======
const CONTRACT_ID = 0xc891361836dbcf588cdb6eb35513b0dcc269991dfa13b5f59ecb0563195d5c09;
>>>>>>> 5a1a9d79c (updating contract ids)

fn main() -> bool {
    let the_abi = abi(Incrementor, CONTRACT_ID);

    let initial = the_abi.get();

    let result = the_abi.increment(5);
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
