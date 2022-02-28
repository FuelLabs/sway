script;

use std::chain::assert;

// 24 bytes
// (8 bytes per elemet)
struct Data {
  one: u64,
  two: u64,
  three: u64,
}

// 24 bytes
// (8 bytes per element)
struct Point {
  x: u8,
  y: u8,
  z: u8,
}

fn return_the_same<T>(elem: T) -> T {
  let x: T = elem;
  x
}

fn main() -> u64 {
    let x = Data {
        one: 1,
        two: 2,
        three: 3,
    };
    let y = Data {
        one: 10000,
        two: 20000,
        three: 30000,
    };
    let p = Point {
      x: 0,
      y: 1,
      z: 2
    };
    let foo = return_the_same(7u64);
    assert(size_of_val(x) == 24);
    assert(size_of_val(y) == 24);
    assert(size_of::<Data>() == 24);
    assert(size_of_val(p) == 24);
    assert(size_of_val(foo) == 8);
    assert(size_of::<Point>() == 24);
    1
}