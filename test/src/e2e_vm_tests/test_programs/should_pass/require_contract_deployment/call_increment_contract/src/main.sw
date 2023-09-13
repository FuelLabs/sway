script;

use increment_abi::Incrementor;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0x30f51fa3c7392dd47565624a4337f96554be9ae1883f0aaa3a298a6711a601ec);
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
