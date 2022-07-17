script;

struct Bar {}

impl Bar {
  fn associated() {}
}

fn main() -> u64 {
  let bar = Bar {};
  bar.associated();
  0
}
