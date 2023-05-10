script;

trait T {
    const ID: u32;
    fn foo() -> u32;
}

struct S {}

impl T for S {
  const ID: u32 = 1;

  fn foo() -> u32 {
    Self::ID
  }
}

fn main() -> u32 {
  let s = S {};
  S::foo()
}
