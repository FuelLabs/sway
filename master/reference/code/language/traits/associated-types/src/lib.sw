script;

trait TypeTrait {
    type T;

    fn method(self, s1: Self::T) -> Self::T;
}

struct Struct {}

struct Struct2 {}

impl TypeTrait for Struct2 {
  type T = Struct;

  fn method(self, s1: Self::T) -> Self::T {
    s1
  }
}

fn main() -> u32 {
  Struct2{}.method(Struct{});

  1
}

