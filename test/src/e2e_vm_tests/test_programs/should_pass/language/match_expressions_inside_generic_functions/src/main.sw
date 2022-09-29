script;

use std::assert::*;
use std::revert::*;

fn third_match<M>(bbbb_value: M) -> u8 {
  match bbbb_value {
    foo => 5u8,
  }
}

fn second_match<L>(bbb_value: L) -> bool {
  match third_match(bbb_value) {
    1u8 => false,
    2u8 => false,
    3u8 => false,
    4u8 => false,
    5u8 => true,
    _ => false,
  }
}

fn first_match<Z>(bb_value: Z) -> u64 {
  match second_match(bb_value) {
    false => 2u64,
    true => 3u64,
  }
}

fn third_if<A>(aaaa_value: A) -> u8 {
  if true {
    5u8
  } else {
    revert(0);
  }
}

fn second_if<Q>(aaa_value: Q) -> bool {
  let third = third_if(aaa_value);
  if third == 1u8 || third == 2u8 || third == 3u8 || third == 4u8 {
    false
  } else if third == 5u8 {
    true
  } else {
    false
  }
}

fn first_if<Y>(aa_value: Y) -> u64 {
  let second = second_if(aa_value);
  if second == false {
    2u64
  } else {
    3u64
  }
}

fn generic_match<W>(cc_value: W) -> u64 {
  match cc_value {
    foo => 3u64,
  }
}

fn generic_if<X>(dd_value: X) -> u64 {
  if true {
    3u64
  } else {
    1u64
  }
}

fn main() -> u64 {

  let a = first_match(true);
  assert(a == 3);

  let b = first_match(1u8);
  assert(b == 3);

  let i = second_if(1u16);
  assert(i);

  let j = second_if(false);
  assert(j);

  let c = first_if(true);
  assert(c == 3);

  let d = first_if(1u8);
  assert(d == 3);

  let e = generic_match(6);
  assert(e == 3);

  let f = generic_match(false);
  assert(f == 3);

  let g = generic_if(6);
  assert(g == 3);

  let h = generic_if(false);
  assert(h == 3);

  1
}