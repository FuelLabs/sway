script;

use increment_abi::Incrementor;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0xb6d07fb970699bc520a53e0534c3d3bdfadfa67e171d2e830d36f65cea014350);
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
