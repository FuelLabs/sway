script;

trait Trait {
    type E;
    const C: Self::E;
}{
    fn get_value() -> Self::E {
      Self::C
    }
}


struct Struct1 {}
struct Struct2 {}

impl Trait for Struct1 {
    type E = u32;
    const C: u32 = 1;
}

impl Trait for Struct2 {
    type E = Struct1;
    const C: Self::E = Struct1 {};
}

fn main() -> u32 {
  let _i: u32 = Struct1::get_value();
  let _c = Struct2::C;
  let _c = Struct2::E::C;

  Struct1::get_value()
}
