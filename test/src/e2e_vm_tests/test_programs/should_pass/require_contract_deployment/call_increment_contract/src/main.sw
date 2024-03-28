script;

use increment_abi::Incrementor;
use dynamic_contract_call::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb7a555aa47aaa778e0b1c351e897e83b08b34a1599114166bb5635721da0ab14;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xf241f685b4c231fa894c4b1c7ad75882d8b0d8292249e7d1e640b001ea15fb49;

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
