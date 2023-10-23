script;

trait TypeTrait {
    type T;
}

struct Struct {
}

impl TypeTrait for Struct {
  type T = u64;
  type T = u32;
}

fn main() -> u32 {
  0
}
