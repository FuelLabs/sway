script;
use basic_storage_abi::StoreU64;

fn main() -> u64 {
    let addr = abi(StoreU64, 0x5ff92b1b01eff2708c9edef22aaae84191680f429b7b12d220632ce5567456d5);
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
