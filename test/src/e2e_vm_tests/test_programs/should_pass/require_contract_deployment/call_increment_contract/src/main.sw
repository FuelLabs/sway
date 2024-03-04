script;

use increment_abi::Incrementor;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xa1aa9555466ef3c61914e5426973e2257cb4dcd8311ffbbe0e8850a9742f312d;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x44c604b49090f343f26f1c27e7433aecf49193165db564a0aa21c361c6d64000;

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

    true
}

fn log(input: u64) {
    asm(r1: input, r2: 42) {
        log r1 r2 r2 r2;
    }
}
