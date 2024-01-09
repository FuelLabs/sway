script;

use std::vec::*;

trait Container {
    type E;
    fn empty() -> Self;
    fn insert(ref mut self, elem: Self::E);
    fn pop_last(ref mut self) -> Option<Self::E>;
}

impl<T> Container for Vec<T> {
    type E = T;
    fn empty() -> Vec<T> { Vec::<T>::new() }
    fn insert(ref mut self, x: T) { self.push(x); }
    fn pop_last(ref mut self) -> Option<T> { self.pop() }
}

fn main() -> u32 {
  let mut s = Vec::<u64>::empty();
  s.insert(1);
  s.insert(2);
  s.insert(3);

  assert_eq(s.pop_last().unwrap(), 3);
  assert_eq(s.pop_last().unwrap(), 2);
  assert_eq(s.pop_last().unwrap(), 1);
  1
}
