script;

trait ConstantId {
    const ID: u32 = 1;
}

struct Struct {}

impl ConstantId for Struct {
  const ID: u32 = 5;
}

fn main() -> u32 {
  Struct::ID
}
