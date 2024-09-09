script;

use std::string::String;

struct A {
   i: u32,
   j: u64,
}

#[inline(never)]
fn foo(a: u64, b: A, c: u64, d: u256, e: b256, f: String, g: [u64; 2], h: b256, i: String, j: u64) -> u64 {
   assert(e == 0x3333333333333333333333333333333333333333333333333333333333333333);
   assert(h == 0x3333333333333333333333333333333333333333333333333333333333333332);
   assert(d == u256::max());
   a + b.j + c + g[0] + j + f.as_bytes().len() + i.as_bytes().len()
}

fn main() -> u64 {
    foo(
          11,
          A { i: 0, j: 2 },
          1,
          u256::max(),
          0x3333333333333333333333333333333333333333333333333333333333333333,
          String::from_ascii_str("hell"),
          [3, 2],
          0x3333333333333333333333333333333333333333333333333333333333333332,
          String::from_ascii_str("world"),
          100,
    )
}
