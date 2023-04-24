script;

abi A {
    const ID: u32 = 1;
    fn foo() -> u32;
}

impl A for Contract {
  const ID: u32 = 2;

  fn foo() -> u32 {
    Self::ID
  }
}

fn main() -> u32 {
  0
}
