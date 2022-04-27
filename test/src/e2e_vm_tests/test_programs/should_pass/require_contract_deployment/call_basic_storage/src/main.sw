script;
use basic_storage_abi::StoreU64;
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(StoreU64, 0x96b98f8f0a5f3c1a72d14974df30df4ba7c5cc904decb776d5ccaa3ab20be3cc);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64(key, value);

    let res = addr.get_u64(key);
    assert(res == value);

    res
}
