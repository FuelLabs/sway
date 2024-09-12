script;

use std::bytes::Bytes;

fn main() -> u64 {
   1
}

#[test]
fn bytes_remove_oob_read() -> u8 {
  let mut a = Bytes::with_capacity(1);
  a.push(1);
  a.remove(0)
}

#[test]
fn vec_remove_oob_read() -> u64 {
  let mut a = Vec::<u64>::with_capacity(2);
  a.push(1);
  a.push(2);
  a.remove(0)
}
