script;

use core::ops::*;
use std::assert::assert;

struct Data<T> {
  x: T
}

impl<T> Data<T> {
  fn contains(self, other: T) -> bool where T: Eq {
    self.x == other
  }
}

fn main() {
  let s = Data { x: 42 };
  assert(s.contains(42));
  assert(!s.contains(41));
}
