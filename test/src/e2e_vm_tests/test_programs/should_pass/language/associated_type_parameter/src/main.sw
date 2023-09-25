script;

trait TypeTrait {
    type T;
}

struct Struct {}

struct Struct2 {}

struct Struct3 {}

impl TypeTrait for Struct2 {
  type T = Struct;
}

impl TypeTrait for Struct3 {
  type T = Struct2;
}

fn func(_s1: Struct2::T, _s2: Struct3::T::T) {

}

fn main() -> u32 {
  func(Struct {}, Struct {});

  1
}
