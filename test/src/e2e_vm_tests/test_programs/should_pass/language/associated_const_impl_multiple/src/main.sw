script;

struct Struct { }

impl Struct {
    const ID: u32 = 1;
}

impl Struct {
    const ID2: u32 = 2;
}

fn main() -> u64 {
  Struct::ID
}
