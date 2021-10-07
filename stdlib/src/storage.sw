library storage;
// These methods will all be replaced by generic functions when those come in.


pub fn store_u64(key: b256, value: u64) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get_u64(key: b256) -> u64 {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: u64
  }
}
