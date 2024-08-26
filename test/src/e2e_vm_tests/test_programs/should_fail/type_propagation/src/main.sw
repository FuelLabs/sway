library;

struct Vec<T> {}

impl<T> Vec<T> {
  fn new() -> Self { return Self {}}
  fn push(self, elem: T) {}
}

trait Wrap {
 fn wrap(self) -> Vec<Self>;
}

impl<T> Wrap for Vec<T> {
  fn wrap(self) -> Vec<Self> {
   let mut v = Vec::new();
   v.push(self);
   v
  }
}

fn main() {
 let mut a: Vec<_> = Vec::new();
 let b: Vec<Vec<_>> = a.wrap();
 let c: Vec<Vec<Vec<u8>>> = b.wrap();
 a.push("str");
}