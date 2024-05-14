script;

trait Trait {
  fn method();
}

#[allow(dead_code)]
struct Struct<T> where T: Trait {
    
}

impl Trait for u64 {
  #[allow(dead_code)]
  fn method() {}
}

#[allow(dead_code)]
const C: Struct<u64> = Struct{};

fn main() -> u64 {
  1
}
