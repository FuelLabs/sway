script;

use increment_abi::Incrementor;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0xa1aa9555466ef3c61914e5426973e2257cb4dcd8311ffbbe0e8850a9742f312d);
    let _ = the_abi.increment(5);
    let _ = the_abi.increment(5);
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
