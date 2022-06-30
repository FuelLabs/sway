script;

use std::assert::assert;

fn main() -> u64 {

  assert(__eq(true, true) == (true == true));
  assert(__eq(true, false) != (true != false));
  assert(__eq(true, true) == (true != false));

  assert(__eq(1, 22) == (1 == 22));
  assert(__eq(1, 1) == (1 == 1));

  let a: u8 = 1;
  let b: u8 = 22;
  let c: u8 = 1;
  assert(__eq(a, b) == (a == b));
  assert(__eq(a, c) == (a == c));

  let a: u16 = 1;
  let b: u16 = 22;
  let c: u16 = 1;
  assert(__eq(a, b) == (a == b));
  assert(__eq(a, c) == (a == c));

  let a: u32 = 1;
  let b: u32 = 22;
  let c: u32 = 1;
  assert(__eq(a, b) == (a == b));
  assert(__eq(a, c) == (a == c));

  let a: u64 = 1;
  let b: u64 = 22;
  let c: u64 = 1;
  assert(__eq(a, b) == (a == b));
  assert(__eq(a, c) == (a == c));

  2
}
