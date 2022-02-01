script;
use basic_storage_abi::{StoreU64, StoreU64Request};

fn main() -> u64 {
  let addr = abi(StoreU64, 0x68a009769e8266282e0b3186602373a0fc65f08c35d260392c8cb12fbcd61277);
  let req = StoreU64Request {
    key: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
    value: 4242
  };

  addr.store_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  let res = addr.get_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req.key);
  res
}

