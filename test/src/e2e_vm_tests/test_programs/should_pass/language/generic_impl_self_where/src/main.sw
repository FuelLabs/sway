script;

dep folder/traits;

use core::ops::*;
use std::assert::assert;
use traits::*;
use traits::nested_traits::*;

struct Data<T> {
  x: T
}

impl<T> Data<T> {
  fn contains(self, other: T) -> bool where T: Eq {
    self.x == other
  }
}

struct Data2<T, K> {
  x: T,
  y: K
}

impl<T,K> Data2<T,K> {
  fn contains(self, other: Self) -> bool where T: Eq, K: Eq  {
    self.x == other.x && self.y == other.y
  }
  fn contains2(self, other: Self) -> bool where T: Eq, K: Eq  {
    self.x == other.x && self.y == other.y
  }
}

impl<T,K> Data2<T,K> {
  fn contains3(self, other: Self) -> bool where T: Eq, K: Eq  {
    self.x == other.x && self.y == other.y
  }

  fn contains4(first: Self, second: Self) -> bool where T: Eq, K: Eq  {
    first.x == second.x && first.y == second.y
  }

  fn contains5(self, other: Self) -> bool where T: MyEq, K: MyEq  {
    self.x.my_eq(other.x) && self.y.my_eq(other.y)
  }

  fn contains6(self, other: Self) -> bool where T: MyEq2, K: MyEq2  {
    self.x.my_eq2(other.x) && self.y.my_eq2(other.y)
  }
}

fn main() {
  let s = Data { x: 42 };
  assert(s.contains(42));
  assert(!s.contains(41));

  let d = Data2 { x: 42, y: 42 };
  assert(d.contains(Data2 { x: 42, y: 42 }));
  assert(!d.contains(Data2 { x: 42, y: 41 }));
  assert(!d.contains(Data2 { x: 41, y: 42 }));

  assert(d.contains2(Data2 { x: 42, y: 42 }));
  assert(!d.contains2(Data2 { x: 42, y: 41 }));
  assert(!d.contains2(Data2 { x: 41, y: 42 }));

  assert(d.contains3(Data2 { x: 42, y: 42 }));
  assert(!d.contains3(Data2 { x: 42, y: 41 }));
  assert(!d.contains3(Data2 { x: 41, y: 42 }));

  assert(Data2::contains4(d, Data2 { x: 42, y: 42 }));
  assert(!Data2::contains4(d, Data2 { x: 42, y: 41 }));
  assert(!Data2::contains4(d, Data2 { x: 41, y: 42 }));

  assert(d.contains5(Data2 { x: 42, y: 42 }));
  assert(!d.contains5(Data2 { x: 42, y: 41 }));
  assert(!d.contains5(Data2 { x: 41, y: 42 }));

  assert(d.contains6(Data2 { x: 42, y: 42 }));
  assert(!d.contains6(Data2 { x: 42, y: 41 }));
  assert(!d.contains6(Data2 { x: 41, y: 42 }));
}
