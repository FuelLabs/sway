script;

trait ConstantId {
    const ID: u32 = 7;
}

struct Struct {}

impl ConstantId for Struct {
  const ID: u32 = 5;
}

impl Struct {
  pub const PUB_ID: u32 = 6;
}

fn main() { }

#[test]
fn test() {
    assert_eq(5, Struct::ID);
    assert_eq(5, <Struct as ConstantId>::ID);
    assert_eq(6, Struct::PUB_ID);
}
