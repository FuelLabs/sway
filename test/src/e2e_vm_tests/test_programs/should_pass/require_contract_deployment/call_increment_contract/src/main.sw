script;

use increment_abi::Incrementor;

fn main() -> bool {
    let the_abi = abi(Incrementor, 0xf5de8211162a13e64a6d868735b62aad9d01836fe0de22d69db1128a69e86bfc);
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
