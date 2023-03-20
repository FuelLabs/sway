script;

trait ConstantId {
    const ID: u32;
} {
  fn foo(self) -> u32 {
    Self::ID
  }
}

struct Struct { }
impl ConstantId for Struct { const ID: u32 = 1; }

fn main() -> u32 {
  let s = Struct {};
  s.foo()
}
