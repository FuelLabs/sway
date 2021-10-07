script;
use basic_storage_abi::StoreU64;
use basic_storage_abi::StoreU64Request;

fn main() -> u64 {
  let addr = abi(StoreU64,0xe1b961f9fe2d690d6734b4fa36fe5341091955c94a9c49447f660d9f33706810);       
  let req = StoreU64Request {
    key: 0,
    value: 42
  };

  addr.store_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  let res = addr.get_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, 0);
  log(res);
  res
}

fn log(input: u64) {
  asm(r1: input, r2: 777777) {
    log r1 r2 r2 r2;
  }
}
