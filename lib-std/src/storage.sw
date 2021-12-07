library storage;
// These methods will all be replaced by generic functions when those come in.
// See https://github.com/FuelLabs/sway/issues/272 for details.

pub fn store<T>(key: b256, value: T) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get<T>(key: b256) -> T {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: T
  }
}
