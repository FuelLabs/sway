script;

use std::u256::U256;
use std::string::String;

struct A {
   i: u32,
   j: u64,
}

#[inline(never)]
fn foo(a: u64, b: A, c: u64, d: U256, e: b256, f: String, g: u64, h: b256, i: String, j: u64) -> u64 {
   a + j
}

fn main() -> u64 {
    foo(
          0,
          A { i: 0, j: 0 },
          1,
          U256::min(),
          0x3333333333333333333333333333333333333333333333333333333333333333,
          String::from_ascii_str("hello"),
          1,
          0x3333333333333333333333333333333333333333333333333333333333333332,
          String::from_ascii_str("world"),
          100,
    )
}
