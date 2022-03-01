script;
use basic_storage_abi::StoreU64;

fn main() -> u64 {
    let addr = abi(StoreU64, 0x68a009769e8266282e0b3186602373a0fc65f08c35d260392c8cb12fbcd61277);
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
