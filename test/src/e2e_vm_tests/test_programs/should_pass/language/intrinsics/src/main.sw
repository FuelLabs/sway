script;

use std::assert::assert;

fn main() -> u64 {

  assert(true == true);
  assert(true != false);

  assert(__eq(1, 22) == (1 == 22));
  assert(__eq(1, 1) == (1 == 1));

  2
}
