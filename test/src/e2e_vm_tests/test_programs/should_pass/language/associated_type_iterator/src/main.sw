script;

trait Iterable {
    type Iter;
    fn iter(self) -> Self::Iter;
}

struct Iter<T> {
    data: Vec<T>,
}

impl<T> Iterable for Vec<T> {
    type Iter = Iter<T>;
    fn iter(self) -> Self::Iter { Iter { data: self } }
}

trait Iterator {
    type Item;
    fn next(ref mut self) -> Option<Self::Item>;
}

impl<T> Iterator for Iter<T> {
    type Item = T;

    fn next(ref mut self) -> Option<Self::Item> {
        if self.data.len() > 0  {
            Some(self.data.remove(0))
        } else {
            None
        }
    }
}

fn main() -> u32 {
  let mut s = Vec::<u64>::new();
  s.push(1);
  s.push(2);
  s.push(3);

  let mut i = s.iter();

  assert_eq(i.next().unwrap(), 1);
  assert_eq(i.next().unwrap(), 2);
  assert_eq(i.next().unwrap(), 3);
  1
}
