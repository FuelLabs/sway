script;
use basic_storage_abi::StoreU64;
use basic_storage_abi::StoreU64Request;

fn main() -> u64 {
  let addr = abi(StoreU64,0xed08bd80d3ef64d32717e8e16e7d3b40fc426d617fbd7494af1ef357a2022909);       
  let req = StoreU64Request {
    key: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
    value: 4242
  };

  addr.store_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req);

  let res = addr.get_u64(10000, 0, 0x0000000000000000000000000000000000000000000000000000000000000000, req.key);
  log(res);
  res
}

fn log(input: u64) {
  asm(r1: input, r2: 777777) {
    log r1 r2 r2 r2;
  }
}
