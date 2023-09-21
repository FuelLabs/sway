script;

trait Trait {
    type E;
    const C: Self::E;
}{
    fn get_value() -> Self::E {
      Self::C
    }
}

impl Trait for u64 {
    type E = u32;
    const C: u32 = 1;
}

impl Trait for u32 {
    type E = u32;
    const C: Self::E = 1;
}

fn main() -> u32 {
  let _i: u32 = u32::get_value();
  u64::get_value()
}
