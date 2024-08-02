script;

struct Struct { }

impl Struct {
    const ID: u32 = 1;
}

impl Struct {
    const ID2: u32 = 2;
}

fn main() {}

#[test]
fn test() {
  assert_eq(1, Struct::ID);
  assert_eq(2, Struct::ID2);
}
