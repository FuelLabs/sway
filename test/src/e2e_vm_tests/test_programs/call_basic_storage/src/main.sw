script;
use basic_storage_abi::StoreU64;

fn main() -> u64 {
    let addr = abi(StoreU64, 0xc664e47a0de686a029134e5122383d99d0d29e54179e14c92dd433413a07620a);
    let key = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    addr.store_u64 {
        gas: 10000, coins: 0, asset_id: 0x0000000000000000000000000000000000000000000000000000000000000000
    }
    (key, value);

    let res = addr.get_u64 {
        gas: 10000, coins: 0, asset_id: 0x0000000000000000000000000000000000000000000000000000000000000000
    }
    (key);
    res
}
