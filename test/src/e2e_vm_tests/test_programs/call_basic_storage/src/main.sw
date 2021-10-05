script;
use basic_storage_abi::StoreU64;
use basic_storage_abi::StoreU64Request;

fn main() -> u64 {
  let addr = abi(StoreU64,0xeeb578f9e1ebfb5b78f8ff74352370c120bc8cacead1f5e4f9c74aafe0ca6bfd);       
  let req = StoreU64Request {
    key: 0,
    value: 42
  };

  addr.store_u64(100, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  addr.get_u64(100, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, 0)
}
