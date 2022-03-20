script;
use basic_storage_abi::StoreU64;

fn main() -> u64 {
    let addr = abi(StoreU64, 0x61c625785e38e1141eb8d6e7e3ef68141b4f44f35250018cd413cb0b1b135bf7);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64 {
        coins: 0,
        asset_id: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
        gas: 10000,
    }
    (key, value);

    let res = addr.get_u64 {
        coins: 0,
        asset_id: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
        gas: 10000
    }
    (key);
    res
}
