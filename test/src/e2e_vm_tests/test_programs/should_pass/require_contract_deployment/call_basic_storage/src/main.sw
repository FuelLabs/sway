script;
use basic_storage_abi::StoreU64;
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(StoreU64, 0x068536389987d7308cd696da40a2f706d2d7ae0889b1df96ca280639289b92c8);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64(key, value);

    let res = addr.get_u64(key);
    assert(res == value);

    res
}
