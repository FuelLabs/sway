script;

struct Struct { }

impl Struct {
    const ID: u32 = 1;
}

fn main() -> u32 {
  const ID: u32 = 1;
  Struct::ID
}
