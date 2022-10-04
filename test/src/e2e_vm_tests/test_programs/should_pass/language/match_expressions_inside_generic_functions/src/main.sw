script;

use std::assert::*;
use std::revert::*;

// fn third_match<A>(value: A) -> u8 {
//   match value {
//     foo => 5u8,
//   }
// }

// fn second_match<B>(value: B) -> bool {
//   match third_match(value) {
//     1u8 => false,
//     2u8 => false,
//     3u8 => false,
//     4u8 => false,
//     5u8 => true,
//     _ => false,
//   }
// }

// fn first_match<C>(value: C) -> u64 {
//   match second_match(value) {
//     false => 2u64,
//     true => 3u64,
//   }
// }

fn third_if<D>(value: D) -> u8 {
  if true {
    5u8
  } else {
    revert(0);
  }
}

fn second_if<E>(value: E) -> bool {
  let third = third_if(value);
  if third == 1u8 || third == 2u8 || third == 3u8 || third == 4u8 {
    false
  } else if third == 5u8 {
    true
  } else {
    false
  }
}

fn first_if<F>(value: F) -> u64 {
  let second = second_if(value);
  if second == false {
    2u64
  } else {
    3u64
  }
}

// fn generic_match<G>(value: G) -> u64 {
//   match value {
//     foo => 3u64,
//   }
// }

// fn generic_if<H>(value: H) -> u64 {
//   if true {
//     3u64
//   } else {
//     1u64
//   }
// }

fn main() -> u64 {
  // let a = first_match(true);
  // assert(a == 3);

  // let b = first_match(1u8);
  // assert(b == 3);

  let c = first_if(true);
  assert(c == 3);

  let d = first_if(1u8);
  assert(d == 3);

  // let e = generic_match(6);
  // assert(e == 3);

  // let f = generic_match(false);
  // assert(f == 3);

  // let g = generic_if(6);
  // assert(g == 3);

  // let h = generic_if(false);
  // assert(h == 3);

  1
}