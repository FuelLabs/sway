script;
use basic_storage_abi::{StoreU64, StoreU64Request};

fn main() -> u64 {
  let addr = abi(StoreU64, 0x410eab113ce1c194952b92295f3d156bce478633feb2e0117360ff28b034a751);
  let req = StoreU64Request {
    key: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
    value: 4242
  };

  addr.store_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  let res = addr.get_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req.key);
  res
}

