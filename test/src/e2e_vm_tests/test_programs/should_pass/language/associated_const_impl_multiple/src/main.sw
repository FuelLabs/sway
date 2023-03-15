script;

struct Struct { }

impl Struct {
    const ID: u32 = 1;
    fn foo() -> u64 { 0 }
}

impl Struct {
    const ID2: u32 = 2;
}

fn main() -> u64 {
  Struct::ID
}
