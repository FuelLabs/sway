library;
pub struct Context {
  pub something: u64
}

impl Context {
  pub fn foo() -> Self {
    Context { something: 10 }
  }
}
