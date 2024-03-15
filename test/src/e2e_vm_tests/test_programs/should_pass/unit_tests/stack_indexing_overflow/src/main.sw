script;

struct M1 {
  a: u256,
  b: u256,
  c: u256,
  d: u256,
}

struct M2 {
  a: M1,
  b: M1,
  c: M1,
  d: M1,
}

const m1: M1 = M1 { a: 0_u256, b: 0_u256, c: 0_u256, d: 0_u256 };
const m2: M2 = M2 { a: m1, b: m1, c: m1, d: m1 };

const MARR : [M2; 6] =
    [
       m2, m2, m2, m2, m2, m2
    ];

fn bar() -> ([M2; 6], [M2; 6]) {
   let mut a = MARR;
   let mut b = MARR;
   a[0].a.b = u256::max();
   (a, b)
}

fn main() -> [M2; 6] {
   let mut b = bar();
   b.0
}

#[test]
fn test() -> [M2; 6] {
   let mut b = bar();
   assert(b.0[0].a.a == 0_u256);
   assert(b.0[0].a.b == u256::max());
   b.0
}
