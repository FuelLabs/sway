script;

use increment_abi::Incrementor;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0xc9baa3acb84a53e91b31b2ecc514d5ad8a715083282dc16d520b5d40a9fc94a7);
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
