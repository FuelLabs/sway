library;

struct Bar {}

impl Bar {
  fn associated() {}
}

pub fn main() {
  let bar = Bar {};
  bar.associated();
}
