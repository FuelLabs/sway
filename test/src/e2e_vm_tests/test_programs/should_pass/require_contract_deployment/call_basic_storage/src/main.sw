script;
use basic_storage_abi::StoreU64;

fn main() -> u64 {
    let addr = abi(StoreU64, 0x3eedeb06664177bd0dea0a1fe0d6e9645c45b8693c902f3a6f67649044f41c9a);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64(key, value);

    let res = addr.get_u64(key);
    res
}
