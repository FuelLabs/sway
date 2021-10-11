script;
use basic_storage_abi::StoreU64;
use basic_storage_abi::StoreU64Request;

fn main() -> u64 {
  let addr = abi(StoreU64,0xa77a026d17532fb95162258a7fb63ad8aee96c4177e81bac6a08561cc49fc11c);       
  let req = StoreU64Request {
    key: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
    value: 4242
  };

  addr.store_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  let res = addr.get_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req.key);
  res
}

