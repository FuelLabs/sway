script;
use basic_storage_abi::StoreU64;
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(StoreU64, 0xee9dd0202c77b119a284bc16b380f5273bbbdfed311de8d581438dc2d7cb00b5);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64(key, value);

    let res = addr.get_u64(key);
    assert(res == value);

    res
}
