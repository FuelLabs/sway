script;
use basic_storage_abi::StoreU64;
use basic_storage_abi::StoreU64Request;

fn main() -> u64 {
  let addr = abi(StoreU64,0xf74fceb7216df5753a1205d69fa9cc0e54c5b22569a1eae891e97d4f1306ce3a);       
  let req = StoreU64Request {
    key: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
    value: 4242
  };

  addr.store_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  let res = addr.get_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req.key);
  res
}

