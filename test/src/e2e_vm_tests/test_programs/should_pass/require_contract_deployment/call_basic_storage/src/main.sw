script;
use basic_storage_abi::StoreU64;
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(StoreU64, 0xf4e12fcac2187e1ac5599476c531560cb6f7aa39bd05d20312a4bd237900b4e4);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64(key, value);

    let res = addr.get_u64(key);
    assert(res == value);

    res
}
