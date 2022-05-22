script;
use basic_storage_abi::StoreU64;
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(StoreU64,0x6b3a8abb373c57d3027acd6c11e83bd6247f55aa4db4545c41506d8ce0b238f8);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64(key, value);

    let res = addr.get_u64(key);
    assert(res == 3*value);

    res
}
