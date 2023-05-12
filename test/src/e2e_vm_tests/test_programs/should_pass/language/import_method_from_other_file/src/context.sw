library;
pub struct Context {
  something: u64
}

impl Context {
  pub fn foo() -> Self {
    Context { something: 10 }
  }
}
