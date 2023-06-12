script;

use increment_abi::Incrementor;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0x1692ebeae2e66a6e0290b39b8d0aeb4c0caeda23ce6f80c7b8d29c318d897bd6);
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
