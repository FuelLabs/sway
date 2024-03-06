script;

use increment_abi::Incrementor;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xa1aa9555466ef3c61914e5426973e2257cb4dcd8311ffbbe0e8850a9742f312d;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xfc785a275345b2344f3dafe03cc910d36691de6456e4d3277d5b2b0fe85479b2;

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
    let call_params = (CONTRACT_ID, 0, 0);
    let coins = 0;
    let asset_id = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let gas = std::registers::global_gas();
    asm(ra: __addr_of(call_params), rb: coins, rc: __addr_of(asset_id), rd: gas) {
        call ra rb rc rd;
    }

    true
}

fn log(input: u64) {
    asm(r1: input, r2: 42) {
        log r1 r2 r2 r2;
    }
}
