script;

trait ConstantId {
    const ID: u32;
}

struct Struct {}

impl ConstantId for Struct {
  const ID: u32 = 1;
}

fn main() -> u32 {
  0
}
