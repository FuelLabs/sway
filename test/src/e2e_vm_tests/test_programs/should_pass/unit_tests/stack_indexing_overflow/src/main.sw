script;

use std::u256::U256;

struct M1 {
  a: U256,
  b: U256,
  c: U256,
  d: U256,
}

struct M2 {
  a: M1,
  b: M1,
  c: M1,
  d: M1,
}

const m1: M1 = M1 { a: U256::new(), b: U256::new(), c: U256::new(), d: U256::new() };
const m2: M2 = M2 { a: m1, b: m1, c: m1, d: m1 };

const MARR : [M2; 6] =
    [
       m2, m2, m2, m2, m2, m2
    ];

fn bar() -> ([M2; 6], [M2; 6]) {
   let mut a = MARR;
   let mut b = MARR;
   a[0].a.b = U256::max();
   (a, b)
}

fn main() -> [M2; 6] {
   let mut b = bar();
   b.0
}

#[test]
fn test() -> [M2; 6] {
   let mut b = bar();
   assert(b.0[0].a.a == U256::new());
   assert(b.0[0].a.b == U256::max());
   b.0
}
