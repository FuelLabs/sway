library context;
struct Context {
  something: u64
}

impl Context {
  fn foo() -> Self {
    Context { something: 10 }
  }
}
