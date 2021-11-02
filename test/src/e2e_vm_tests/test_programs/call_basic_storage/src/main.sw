script;
use basic_storage_abi::StoreU64;
use basic_storage_abi::StoreU64Request;

fn main() -> u64 {
  let addr = abi(StoreU64, 0x145be2230354cac71a59580f00793ec67d2789b983025e1107867560360e007b);       
  let req = StoreU64Request {
    key: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
    value: 4242
  };

  addr.store_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  let res = addr.get_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req.key);
  res
}

